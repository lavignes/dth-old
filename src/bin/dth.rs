use dth::{
    self,
    math::Vector3,
    math::{Matrix4, Vector2},
    util::BoxedError,
};
use futures::executor;
use sdl2::{
    event::{Event, WindowEvent},
    video::Window,
    Sdl,
};
use wgpu::{
    util::BufferInitDescriptor, util::DeviceExt, BackendBit, BufferUsage, Color,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits,
    LoadOp, Operations, PowerPreference, PresentMode, Queue, RenderPassColorAttachmentDescriptor,
    RenderPassDepthStencilAttachmentDescriptor, RenderPassDescriptor, RequestAdapterOptions,
    Surface, SwapChain, SwapChainDescriptor, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsage, TextureView, TextureViewDescriptor,
};

use log::LevelFilter;
use std::time::{Duration, Instant};

/// The smallest possible push-constant buffer size (in bytes) according to WGPU docs.
/// This is the lower limit for push-constants on Vulkan.
const MAX_PUSH_CONSTANT_SIZE: usize = 128;

fn setup_rendering(sdl: &Sdl, size: Vector2) -> Result<(WindowTarget, Device, Queue), BoxedError> {
    let sdl_video = sdl.video()?;
    let window = sdl_video
        .window("dth", size.x() as u32, size.y() as u32)
        .resizable()
        .build()?;
    let instance = Instance::new(BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    // TODO: convert to plain ? when try_trait it stable
    let adapter = executor::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .ok_or("Failed to request GFX adapter")?;

    let (device, queue) = executor::block_on(adapter.request_device(
        &DeviceDescriptor {
            features: Features::PUSH_CONSTANTS,
            limits: Limits {
                max_push_constant_size: MAX_PUSH_CONSTANT_SIZE as u32,
                ..Limits::default()
            },
            shader_validation: true,
        },
        None,
    ))?;
    Ok((
        WindowTarget::new(&device, window, surface, size.into()),
        device,
        queue,
    ))
}

struct WindowTarget {
    pub window: Window,
    pub surface: Surface,
    pub swap_chain: SwapChain,
    pub depth_buffer: TextureView,
}

impl WindowTarget {
    fn new(device: &Device, window: Window, surface: Surface, size: (u32, u32)) -> WindowTarget {
        let swap_chain = WindowTarget::create_swap_chain(&device, &surface, size);
        let depth_buffer = WindowTarget::create_depth_buffer(&device, size);
        WindowTarget {
            window,
            surface,
            swap_chain,
            depth_buffer,
        }
    }

    fn size(&self) -> Vector2 {
        self.window.size().into()
    }

    fn synchronize_size(&mut self, device: &Device, size: (u32, u32)) {
        self.swap_chain = WindowTarget::create_swap_chain(&device, &self.surface, size);
        self.depth_buffer = WindowTarget::create_depth_buffer(&device, size);
    }

    fn create_swap_chain(device: &Device, surface: &Surface, size: (u32, u32)) -> SwapChain {
        device.create_swap_chain(
            &surface,
            &SwapChainDescriptor {
                usage: TextureUsage::OUTPUT_ATTACHMENT,
                format: TextureFormat::Bgra8Unorm,
                width: size.0,
                height: size.1,
                // v-sync
                present_mode: PresentMode::Immediate,
            },
        )
    }

    fn create_depth_buffer(device: &Device, size: (u32, u32)) -> TextureView {
        device
            .create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: size.0,
                    height: size.1,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&TextureViewDescriptor::default())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Projection(pub Matrix4);

unsafe impl bytemuck::Zeroable for Projection {}

unsafe impl bytemuck::Pod for Projection {}

impl Projection {
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct View(pub Matrix4);

unsafe impl bytemuck::Zeroable for View {}

unsafe impl bytemuck::Pod for View {}

impl View {
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

fn create_projection(size: Vector2) -> Projection {
    Projection(
        &Matrix4::perspective(1.0, size.x() / size.y(), 0.1, 60000.0)
            * &Matrix4::vulkan_projection_correct(),
    )
}

fn main() -> Result<(), BoxedError> {
    env_logger::builder()
        .filter_level(LevelFilter::Error)
        .filter_module("dth", LevelFilter::Debug)
        .init();

    let sdl = sdl2::init()?;
    let mut event_pump = sdl.event_pump()?;
    let (mut target, device, queue) = setup_rendering(&sdl, (800, 600).into())?;

    let projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: create_projection(target.size()).to_bytes(),
        usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    });

    let _view_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: View(Matrix4::look_at(
            (0.0, 10.0, -20.0).into(),
            Vector3::splat(0.0),
            Vector3::up(),
        ))
        .to_bytes(),
        usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    });

    let mut frame_rate_timer = Instant::now();
    let mut frame_rate = 0;
    let mut update_timer = Instant::now();
    let mut update_delta_time = 0.0;
    let update_rate = Duration::from_secs_f32(1.0 / 60.0);

    'running: loop {
        let mut projection_dirty = None;
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(w, h) = win_event {
                        target.synchronize_size(&device, (w as u32, h as u32));
                        projection_dirty = Some((w, h).into());
                    }
                }
                _ => {}
            }
        }

        // Fixed update
        update_delta_time += update_timer.elapsed().as_secs_f32();
        update_timer = Instant::now();

        while update_delta_time > update_rate.as_secs_f32() {
            update_delta_time -= update_rate.as_secs_f32();
        }

        // The render buffers will automatically be swapped when this texture drops
        let current_frame = target.swap_chain.get_current_frame()?;

        if let Some(size) = projection_dirty {
            queue.write_buffer(&projection_buffer, 0, create_projection(size).to_bytes());
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &current_frame.output.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.678,
                            g: 0.847,
                            b: 0.902,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &target.depth_buffer,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }
        queue.submit(Some(encoder.finish()));

        frame_rate += 1;
        if frame_rate_timer.elapsed() >= Duration::from_secs(1) {
            target
                .window
                .set_title(&format!("dth fps: {}", frame_rate))?;
            frame_rate = 0;
            frame_rate_timer = Instant::now();
        }
    }

    Ok(())
}
