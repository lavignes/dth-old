use futures::executor;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::Window,
    Sdl,
};
use wgpu::{
    util::BufferInitDescriptor, util::DeviceExt, BackendBit, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendDescriptor,
    BlendFactor, BlendOperation, BufferUsage, Color, ColorStateDescriptor, ColorWrite,
    CommandEncoderDescriptor, CompareFunction, CullMode, DepthStencilStateDescriptor, Device,
    DeviceDescriptor, Extent3d, Features, FrontFace, IndexFormat, InputStepMode, Instance, Limits,
    LoadOp, Operations, PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveTopology,
    ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule,
    ShaderStage, StencilStateDescriptor, StencilStateFaceDescriptor, Surface, SwapChain,
    SwapChainDescriptor, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
    TextureView, TextureViewDescriptor, VertexBufferDescriptor, VertexStateDescriptor,
};

use dth::{
    self,
    gfx::Frustum,
    gfx::{ChunkMesher, Mesh, Vertex},
    math::Quaternion,
    math::Vector3,
    math::{self, Matrix4, Vector2},
    util::{self, BoxedError},
    world::Chunk,
};
use log::LevelFilter;
use std::{
    f32,
    io::Read,
    mem,
    num::NonZeroU64,
    path::Path,
    time::{Duration, Instant},
};

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

    #[inline]
    fn size(&self) -> Vector2 {
        self.window.size().into()
    }

    #[inline]
    fn aspect_ratio(&self) -> f32 {
        let size = self.size();
        size.x() / size.y()
    }

    #[inline]
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
                // present_mode: PresentMode::Fifo,
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
        &Matrix4::perspective(1.0, size.x() / size.y(), 0.001, 60000.0)
            * &Matrix4::vulkan_projection_correct(),
    )
}

fn create_shader_module<P: AsRef<Path>>(
    device: &Device,
    path: P,
) -> Result<ShaderModule, BoxedError> {
    let mut spirv_buffer = Vec::new();
    util::buf_open(path)?.read_to_end(&mut spirv_buffer)?;
    Ok(device.create_shader_module(wgpu::util::make_spirv(spirv_buffer.as_slice())))
}

fn main() -> Result<(), BoxedError> {
    env_logger::builder()
        .filter_level(LevelFilter::Error)
        .filter_module("dth", LevelFilter::Debug)
        .init();

    let sdl = sdl2::init()?;
    let mut event_pump = sdl.event_pump()?;
    let (mut target, device, queue) = setup_rendering(&sdl, (800, 600).into())?;

    let mut view = View(Matrix4::identity());
    let mut projection = create_projection(target.size());

    let projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: projection.to_bytes(),
        usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    });

    let view_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: view.to_bytes(),
        usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    });

    let basic_chunk_vs = create_shader_module(&device, "res/shaders/basic_chunk_wgpu.vert.spv")?;
    let basic_chunk_fs = create_shader_module(&device, "res/shaders/basic_chunk_wgpu.frag.spv")?;

    let basic_chunk_primary_bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // projection
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: NonZeroU64::new(mem::size_of::<Projection>() as u64),
                    },
                    count: None,
                },
                // view
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: NonZeroU64::new(mem::size_of::<View>() as u64),
                    },
                    count: None,
                },
            ],
        });

    let basic_chunk_primary_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &basic_chunk_primary_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(projection_buffer.slice(..)),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(view_buffer.slice(..)),
            },
        ],
    });

    let basic_chunk_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&basic_chunk_primary_bind_group_layout],
        push_constant_ranges: &[],
    });

    let basic_chunk_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&basic_chunk_pipeline_layout),
        vertex_stage: ProgrammableStageDescriptor {
            module: &basic_chunk_vs,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &basic_chunk_fs,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Cw,
            cull_mode: CullMode::Back,
            clamp_depth: false,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        // primitive_topology: PrimitiveTopology::LineList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Bgra8Unorm,
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilStateDescriptor {
                front: StencilStateFaceDescriptor::IGNORE,
                back: StencilStateFaceDescriptor::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
        }),
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[VertexBufferDescriptor {
                stride: mem::size_of::<Vertex>() as u64,
                step_mode: InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float3],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut mesher = ChunkMesher::default();
    let mut meshes = Vec::with_capacity(64 * 64);
    let mut chunks = Vec::with_capacity(64 * 64);

    let mut mesh_vertex_buffers = Vec::with_capacity(64 * 64);
    let mut mesh_index_buffers = Vec::with_capacity(64 * 64);

    for z in 0..64 {
        for x in 0..64 {
            let mut chunk = Chunk::randomized();
            let mut mesh = Mesh::default();
            chunk.set_position((x as f32 * 16.0, 0.0, z as f32 * 16.0).into());
            mesher.greedy(&chunk, &mut mesh);
            mesh_vertex_buffers.push(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(mesh.vertices().as_slice()),
                usage: BufferUsage::VERTEX,
            }));
            mesh_index_buffers.push(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(mesh.indices().as_slice()),
                usage: BufferUsage::INDEX,
            }));
            chunks.push(chunk);
            meshes.push(mesh);
        }
    }

    let mut frame_rate_timer = Instant::now();
    let mut frame_rate = 0;
    let mut update_timer = Instant::now();
    let mut update_delta_time = 0.0;
    let update_rate = Duration::from_secs_f32(1.0 / 60.0);

    let mut mouse_pos = Vector2::default();
    let mut camera_euler_angles = Vector2::new(4.0, 0.0);
    let mut camera_position = Vector3::new(-16.0, 8.0, -16.0);
    // TODO: not synced with projection creation (maybe make a new struct of projection components)
    let mut frustum = Frustum::new(
        1.0,
        target.aspect_ratio(),
        0.001,
        60000.0,
        camera_position,
        camera_position - Vector3::forward(),
        Vector3::up(),
    );

    let mut w = false;
    let mut s = false;
    let mut a = false;
    let mut d = false;
    let mut l_shift = false;
    let mut space = false;

    'running: loop {
        let mut projection_dirty = None;
        let mut mouse_dirty = false;
        let mut physics_dirty = false;

        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::MouseMotion { x, y, .. } => {
                    mouse_pos = (x, y).into();
                    mouse_dirty = true;
                }
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(w, h) = win_event {
                        target.synchronize_size(&device, (w as u32, h as u32));
                        projection_dirty = Some((w, h).into());
                    }
                }
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Q) => {
                        for (i, chunk) in chunks.iter_mut().enumerate() {
                            chunk.randomize();
                            let mesh = &mut meshes[i];
                            mesher.greedy(&chunk, mesh);
                            mesh_vertex_buffers[i] =
                                device.create_buffer_init(&BufferInitDescriptor {
                                    label: None,
                                    contents: bytemuck::cast_slice(mesh.vertices().as_slice()),
                                    usage: BufferUsage::VERTEX,
                                });
                            mesh_index_buffers[i] =
                                device.create_buffer_init(&BufferInitDescriptor {
                                    label: None,
                                    contents: bytemuck::cast_slice(mesh.indices().as_slice()),
                                    usage: BufferUsage::INDEX,
                                });
                        }
                    }
                    Some(Keycode::W) => w = true,
                    Some(Keycode::S) => s = true,
                    Some(Keycode::A) => a = true,
                    Some(Keycode::D) => d = true,
                    Some(Keycode::LShift) => l_shift = true,
                    Some(Keycode::Space) => space = true,
                    _ => (),
                },
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(Keycode::W) => w = false,
                    Some(Keycode::S) => s = false,
                    Some(Keycode::A) => a = false,
                    Some(Keycode::D) => d = false,
                    Some(Keycode::LShift) => l_shift = false,
                    Some(Keycode::Space) => space = false,
                    _ => (),
                },
                _ => {}
            }
        }

        if mouse_dirty {
            let size = target.size();
            let center = size / 2.0;
            let delta = mouse_pos - center;
            camera_euler_angles = Vector2::new(
                math::normalize_angle(camera_euler_angles.x() + delta.x() * 0.002),
                math::normalize_angle(camera_euler_angles.y() + -delta.y() * 0.002),
            );
            sdl.mouse()
                .warp_mouse_in_window(&target.window, center.x() as i32, center.y() as i32);
        }

        // Fixed update
        update_delta_time += update_timer.elapsed().as_secs_f32();
        update_timer = Instant::now();
        while update_delta_time > update_rate.as_secs_f32() {
            update_delta_time -= update_rate.as_secs_f32();
            physics_dirty = true;

            // TODO: These should add velocity instead
            if w {
                let theta = camera_euler_angles.x();
                camera_position -= (theta.sin(), 0.0, theta.cos()).into();
            } else if s {
                let theta = camera_euler_angles.x();
                camera_position += (theta.sin(), 0.0, theta.cos()).into();
            }

            if a {
                let theta = camera_euler_angles.x() + f32::consts::FRAC_PI_2;
                camera_position += (theta.sin(), 0.0, theta.cos()).into();
            } else if d {
                let theta = camera_euler_angles.x() - f32::consts::FRAC_PI_2;
                camera_position += (theta.sin(), 0.0, theta.cos()).into();
            }

            if space {
                camera_position += (0.0, 1.0, 0.0).into();
            } else if l_shift {
                camera_position += (0.0, -1.0, 0.0).into();
            }
        }

        if mouse_dirty || physics_dirty {
            let camera_quaternion = Quaternion::from_angle_up(camera_euler_angles.x())
                * Quaternion::from_angle_right(camera_euler_angles.y());

            // Here we create a unit vector from the camera in the direction of the camera angle
            // I don't understand exactly why the rotation quaternion is "backward"
            let at = camera_position - camera_quaternion.forward_axis();
            // Then we can pass it to the handy look at matrix
            view = View(Matrix4::look_at(camera_position, at, Vector3::up()));

            queue.write_buffer(&view_buffer, 0, view.to_bytes());
            frustum.update_look_at(camera_position, at, Vector3::up());
        }

        // The render buffers will automatically be swapped when this texture drops
        let current_frame = target.swap_chain.get_current_frame()?;

        if let Some(size) = projection_dirty {
            projection = create_projection(size);
            queue.write_buffer(&projection_buffer, 0, projection.to_bytes());
            frustum.update_projection(1.0, target.aspect_ratio(), 0.001, 60000.0);
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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

            render_pass.set_pipeline(&basic_chunk_pipeline);
            render_pass.set_bind_group(0, &basic_chunk_primary_bind_group, &[]);

            for (((chunk, mesh), mesh_vertex_buffer), mesh_index_buffer) in chunks
                .iter()
                .zip(&meshes)
                .zip(&mesh_vertex_buffers)
                .zip(&mesh_index_buffers)
            {
                if !frustum.infinite_cylinder_inside(chunk.position() + Vector3::splat(8.0), 16.0) {
                    continue;
                }
                render_pass.set_vertex_buffer(0, mesh_vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh_index_buffer.slice(..));
                render_pass.draw_indexed(0..mesh.indices().len() as u32, 0, 0..1);
            }
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
