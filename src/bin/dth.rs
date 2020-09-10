use futures::executor;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::Window,
    Sdl,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BackendBit, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendDescriptor,
    BlendFactor, BlendOperation, Buffer, BufferUsage, Color, ColorStateDescriptor, ColorWrite,
    CommandEncoderDescriptor, CompareFunction, CullMode, DepthStencilStateDescriptor, Device,
    DeviceDescriptor, Extent3d, Features, FilterMode, FrontFace, IndexFormat, InputStepMode,
    Instance, Limits, LoadOp, Maintain, Operations, Origin3d, PipelineLayoutDescriptor,
    PowerPreference, PresentMode, PrimitiveTopology, ProgrammableStageDescriptor,
    PushConstantRange, Queue, RasterizationStateDescriptor, RenderPassColorAttachmentDescriptor,
    RenderPassDepthStencilAttachmentDescriptor, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerDescriptor, ShaderModule, ShaderStage,
    StencilStateDescriptor, StencilStateFaceDescriptor, Surface, SwapChain, SwapChainDescriptor,
    Texture, TextureAspect, TextureComponentType, TextureCopyView, TextureDataLayout,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsage, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexBufferDescriptor, VertexStateDescriptor,
};

use dth::{
    self,
    gfx::{
        Bitmap, BitmapFormat, BitmapReader, ColladaReader, Frustum, PerspectiveProjection,
        StaticMaterialMesh, StaticMaterialVertex, Transform,
    },
    math::{self, Matrix3, Matrix4, Quaternion, Vector2, Vector3},
    util::{self, BoxedError},
};
use log::LevelFilter;
use rand::Rng;
use std::thread::Thread;
use std::{
    f32,
    io::Read,
    mem,
    num::NonZeroU64,
    panic,
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
            features: Features::PUSH_CONSTANTS
                | Features::SAMPLED_TEXTURE_ARRAY_DYNAMIC_INDEXING
                | Features::TEXTURE_COMPRESSION_BC,
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
    window: Window,
    surface: Surface,
    swap_chain: SwapChain,
    hdr_buffer: TextureView,
    bloom_buffer: TextureView,
    ping_pong_buffers: [TextureView; 2],
    depth_buffer: TextureView,
}

impl WindowTarget {
    fn new(device: &Device, window: Window, surface: Surface, size: (u32, u32)) -> WindowTarget {
        let swap_chain = WindowTarget::create_swap_chain(&device, &surface, size);
        let hdr_buffer = WindowTarget::create_hdr_frame_buffer(&device, size);
        let bloom_buffer = WindowTarget::create_hdr_frame_buffer(&device, size);
        let ping_pong_buffers = [
            WindowTarget::create_hdr_frame_buffer(&device, size),
            WindowTarget::create_hdr_frame_buffer(&device, size),
        ];
        let depth_buffer = WindowTarget::create_depth_buffer(&device, size);
        WindowTarget {
            window,
            surface,
            swap_chain,
            hdr_buffer,
            bloom_buffer,
            ping_pong_buffers,
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
        self.hdr_buffer = WindowTarget::create_hdr_frame_buffer(&device, size);
        self.bloom_buffer = WindowTarget::create_hdr_frame_buffer(&device, size);
        self.ping_pong_buffers = [
            WindowTarget::create_hdr_frame_buffer(&device, size),
            WindowTarget::create_hdr_frame_buffer(&device, size),
        ];
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

    fn create_hdr_frame_buffer(device: &Device, size: (u32, u32)) -> TextureView {
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
                format: TextureFormat::Rgba16Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
            })
            .create_view(&TextureViewDescriptor::default())
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
            .create_view(&TextureViewDescriptor {
                aspect: TextureAspect::DepthOnly,
                ..TextureViewDescriptor::default()
            })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct Projection(Matrix4);

unsafe impl bytemuck::Zeroable for Projection {}

unsafe impl bytemuck::Pod for Projection {}

impl Projection {
    #[inline]
    fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct View {
    view: Matrix4,
    view_position: Vector3,
}

unsafe impl bytemuck::Zeroable for View {}

unsafe impl bytemuck::Pod for View {}

impl View {
    fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct Exposure(f32);

unsafe impl bytemuck::Zeroable for Exposure {}

unsafe impl bytemuck::Pod for Exposure {}

impl Exposure {
    #[inline]
    fn to_words(&self) -> &[u32] {
        bytemuck::cast_slice(bytemuck::bytes_of(self))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct GaussianBlur {
    horizontal: u32,
    weights: [f32; 5],
}

unsafe impl bytemuck::Zeroable for GaussianBlur {}

unsafe impl bytemuck::Pod for GaussianBlur {}

impl GaussianBlur {
    #[inline]
    fn to_words(&self) -> &[u32] {
        bytemuck::cast_slice(bytemuck::bytes_of(self))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct StaticMaterialMeshModel {
    model: Matrix4,
    inverse_normal: Matrix3,
    tex_index: u32,
}

unsafe impl bytemuck::Zeroable for StaticMaterialMeshModel {}

unsafe impl bytemuck::Pod for StaticMaterialMeshModel {}

impl StaticMaterialMeshModel {
    #[inline]
    fn to_words(&self) -> &[u32] {
        bytemuck::cast_slice(bytemuck::bytes_of(self))
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct OutputTargetVertex {
    position: Vector3,
    tex_coord: Vector2,
}

unsafe impl bytemuck::Zeroable for OutputTargetVertex {}

unsafe impl bytemuck::Pod for OutputTargetVertex {}

const OUTPUT_TARGET_VERTICES: [OutputTargetVertex; 6] = [
    OutputTargetVertex {
        position: Vector3::new(-1.0, -1.0, 0.0),
        tex_coord: Vector2::new(0.0, 1.0),
    },
    OutputTargetVertex {
        position: Vector3::new(-1.0, 1.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
    },
    OutputTargetVertex {
        position: Vector3::new(1.0, -1.0, 0.0),
        tex_coord: Vector2::new(1.0, 1.0),
    },
    OutputTargetVertex {
        position: Vector3::new(1.0, -1.0, 0.0),
        tex_coord: Vector2::new(1.0, 1.0),
    },
    OutputTargetVertex {
        position: Vector3::new(-1.0, 1.0, 0.0),
        tex_coord: Vector2::new(0.0, 0.0),
    },
    OutputTargetVertex {
        position: Vector3::new(1.0, 1.0, 0.0),
        tex_coord: Vector2::new(1.0, 0.0),
    },
];

#[inline]
fn compute_projection(projection: &PerspectiveProjection) -> Projection {
    Projection(&Matrix4::perspective(projection) * &Matrix4::vulkan_projection_correct())
}

#[inline]
fn compute_view(camera_euler_angles: Vector2, camera_position: Vector3) -> (View, Vector3) {
    let camera_quaternion = Quaternion::from_angle_up(camera_euler_angles.x())
        * Quaternion::from_angle_right(camera_euler_angles.y());

    // Here we create a unit vector from the camera in the direction of the camera angle
    // I don't understand exactly why the rotation quaternion is "backward"
    let at = camera_position - camera_quaternion.forward_axis();

    // Then we can pass it to the handy look at matrix
    (
        View {
            view: Matrix4::look_at(camera_position, at, Vector3::up()),
            view_position: camera_position,
        },
        at,
    )
}

#[derive(Debug)]
struct TextureManager {
    resolution: usize,
    depth: usize,
    texture_index: u32,
    diffuse_maps: (Texture, TextureView),
    specular_maps: (Texture, TextureView),
    emmisive_maps: (Texture, TextureView),
    normal_maps: (Texture, TextureView),
}

impl TextureManager {
    pub fn new(device: &Device, resolution: usize, depth: usize) -> TextureManager {
        TextureManager {
            resolution,
            depth,
            texture_index: 0,
            diffuse_maps: TextureManager::create_texture(
                device,
                resolution,
                texture_format_from_bitmap_format(BitmapFormat::DXT5),
                depth,
                8,
            ),
            specular_maps: TextureManager::create_texture(
                device,
                resolution,
                texture_format_from_bitmap_format(BitmapFormat::GrayU8),
                depth,
                8,
            ),
            emmisive_maps: TextureManager::create_texture(
                device,
                resolution,
                texture_format_from_bitmap_format(BitmapFormat::GrayU8),
                depth,
                8,
            ),
            normal_maps: TextureManager::create_texture(
                device,
                resolution,
                texture_format_from_bitmap_format(BitmapFormat::DXT1),
                depth,
                8,
            ),
        }
    }

    pub fn load_texture(
        &mut self,
        queue: &Queue,
        diffuse: Bitmap,
        normal: Bitmap,
        specular: Bitmap,
        emissive: Bitmap,
    ) -> Result<u32, BoxedError> {
        let texture_index = self.texture_index;
        self.texture_index += 1;

        TextureManager::write_texture(queue, &self.diffuse_maps.0, texture_index, &diffuse, 8);
        TextureManager::write_texture(queue, &self.normal_maps.0, texture_index, &normal, 8);
        TextureManager::write_texture(queue, &self.specular_maps.0, texture_index, &specular, 8);
        TextureManager::write_texture(queue, &self.emmisive_maps.0, texture_index, &emissive, 8);

        Ok(texture_index)
    }

    fn create_texture(
        device: &Device,
        resolution: usize,
        format: TextureFormat,
        depth: usize,
        mip_levels: usize,
    ) -> (Texture, TextureView) {
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: resolution as u32,
                height: resolution as u32,
                depth: depth as u32,
            },
            mip_level_count: mip_levels as u32,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        (texture, texture_view)
    }

    fn write_texture(
        queue: &Queue,
        texture: &Texture,
        index: u32,
        bitmap: &Bitmap,
        mip_levels: usize,
    ) {
        for mip_level in 0..mip_levels {
            let size = bitmap.mip_size(mip_level);
            queue.write_texture(
                TextureCopyView {
                    texture: &texture,
                    mip_level: mip_level as u32,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: index,
                    },
                },
                bitmap.mip_data(mip_level),
                TextureDataLayout {
                    offset: 0,
                    bytes_per_row: bitmap.mip_bytes_per_row(mip_level) as u32,
                    rows_per_image: size.y() as u32,
                },
                Extent3d {
                    width: size.x() as u32,
                    height: size.y() as u32,
                    depth: 1,
                },
            );
        }
    }
}

#[inline]
fn texture_format_from_bitmap_format(format: BitmapFormat) -> TextureFormat {
    match format {
        BitmapFormat::BgraU8 => TextureFormat::Bgra8Unorm,
        BitmapFormat::GrayU8 => TextureFormat::R8Unorm,
        BitmapFormat::DXT1 => TextureFormat::Bc1RgbaUnorm,
        BitmapFormat::DXT3 => TextureFormat::Bc2RgbaUnorm,
        BitmapFormat::DXT5 => TextureFormat::Bc3RgbaUnorm,
    }
}

fn load_texture(
    device: &Device,
    queue: &Queue,
    bitmap: &Bitmap,
) -> Result<TextureView, BoxedError> {
    let size = (bitmap.size().x() as u32, bitmap.size().y() as u32);
    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: size.0,
            height: size.1,
            depth: 256,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: texture_format_from_bitmap_format(bitmap.format()),
        usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
    });
    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    queue.write_texture(
        TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
        },
        bitmap.data(),
        TextureDataLayout {
            offset: 0,
            bytes_per_row: bitmap.bytes_per_row() as u32,
            rows_per_image: size.1,
        },
        Extent3d {
            width: size.0,
            height: size.1,
            depth: 1,
        },
    );
    Ok(texture_view)
}

fn load_shader<P: AsRef<Path>>(device: &Device, path: P) -> Result<ShaderModule, BoxedError> {
    let mut buffer = Vec::new();
    util::buf_open(path)?.read_to_end(&mut buffer)?;
    Ok(device.create_shader_module(wgpu::util::make_spirv(buffer.as_slice())))
}

fn create_color_state(format: TextureFormat) -> ColorStateDescriptor {
    ColorStateDescriptor {
        format,
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
    }
}

fn create_forward_primary_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    sampler: &Sampler,
    hdr_buffer: &TextureView,
    blur_buffer: &TextureView,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(hdr_buffer),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(blur_buffer),
            },
        ],
    })
}

fn create_blur_primary_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    sampler: &Sampler,
    image: &TextureView,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Sampler(sampler),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(image),
            },
        ],
    })
}

fn main_real() -> Result<(), BoxedError> {
    let sdl = sdl2::init()?;
    let mut event_pump = sdl.event_pump()?;
    let (mut target, device, queue) = setup_rendering(&sdl, (800, 600).into())?;

    let mut projection = PerspectiveProjection {
        fov: 1.0,
        aspect_ratio: target.aspect_ratio(),
        near: 0.001,
        far: 60000.0,
    };

    let mut mouse_pos = Vector2::default();
    let mut camera_euler_angles = Vector2::new(0.0, 0.0);
    let mut camera_position = Vector3::new(-16.0, 8.0, -16.0);
    let view_parts = compute_view(camera_euler_angles, camera_position);
    let mut frustum = Frustum::new(&projection, camera_position, view_parts.1, Vector3::up());

    let projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: compute_projection(&projection).to_bytes(),
        usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    });

    let view_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: view_parts.0.to_bytes(),
        usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    });

    let output_target_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&OUTPUT_TARGET_VERTICES),
        usage: BufferUsage::VERTEX,
    });

    let static_material_vs = load_shader(&device, "res/shaders/static_material.vert.glsl.spv")?;
    let static_material_fs = load_shader(&device, "res/shaders/static_material.frag.glsl.spv")?;

    let static_material_primary_bind_group_layout =
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
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: NonZeroU64::new(mem::size_of::<View>() as u64),
                    },
                    count: None,
                },
                // sampler0
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
        });

    let static_material_texture_bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // diffuse_map
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
                // specular_map
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
                // emissive_map
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
                // normal_map
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
            ],
        });

    let basic_sampler = device.create_sampler(&SamplerDescriptor {
        label: None,
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        lod_min_clamp: 0.0,
        lod_max_clamp: 1.0,
        compare: None,
        anisotropy_clamp: None,
    });

    let static_material_primary_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &static_material_primary_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(projection_buffer.slice(..)),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(view_buffer.slice(..)),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(&basic_sampler),
            },
        ],
    });

    let static_material_pipeline_layout =
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &static_material_primary_bind_group_layout,
                &static_material_texture_bind_group_layout,
            ],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                range: 0..mem::size_of::<StaticMaterialMeshModel>() as u32,
            }],
        });

    let static_material_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&static_material_pipeline_layout),
        vertex_stage: ProgrammableStageDescriptor {
            module: &static_material_vs,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &static_material_fs,
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
        color_states: &[
            create_color_state(TextureFormat::Rgba16Float),
            create_color_state(TextureFormat::Rgba16Float),
        ],
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
                stride: mem::size_of::<StaticMaterialVertex>() as u64,
                step_mode: InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float2, 3 => Uint],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let blur_vs = load_shader(&device, "res/shaders/blur.vert.glsl.spv")?;
    let blur_fs = load_shader(&device, "res/shaders/blur.frag.glsl.spv")?;

    let blur_primary_bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // sampler0
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                    count: None,
                },
                // image
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
            ],
        });

    let blur_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&blur_primary_bind_group_layout],
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStage::FRAGMENT,
            range: 0..mem::size_of::<GaussianBlur>() as u32,
        }],
    });

    let blur_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&blur_pipeline_layout),
        vertex_stage: ProgrammableStageDescriptor {
            module: &blur_vs,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &blur_fs,
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
        color_states: &[create_color_state(TextureFormat::Rgba16Float)],
        depth_stencil_state: None,
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[VertexBufferDescriptor {
                stride: mem::size_of::<OutputTargetVertex>() as u64,
                step_mode: InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float2],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let hdr_vs = load_shader(&device, "res/shaders/hdr.vert.glsl.spv")?;
    let hdr_fs = load_shader(&device, "res/shaders/hdr.frag.glsl.spv")?;

    let forward_primary_bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // sampler0
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                    count: None,
                },
                // hdr_buffer
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
                // blur_buffer
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                    },
                    count: None,
                },
            ],
        });

    let forward_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&forward_primary_bind_group_layout],
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStage::FRAGMENT,
            range: 0..mem::size_of::<Exposure>() as u32,
        }],
    });

    let forward_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&forward_pipeline_layout),
        vertex_stage: ProgrammableStageDescriptor {
            module: &hdr_vs,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &hdr_fs,
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
        color_states: &[create_color_state(TextureFormat::Bgra8Unorm)],
        depth_stencil_state: None,
        vertex_state: VertexStateDescriptor {
            index_format: IndexFormat::Uint32,
            vertex_buffers: &[VertexBufferDescriptor {
                stride: mem::size_of::<OutputTargetVertex>() as u64,
                step_mode: InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float2],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut blur_primary_bind_groups = [
        create_blur_primary_bind_group(
            &device,
            &blur_primary_bind_group_layout,
            &basic_sampler,
            &target.bloom_buffer,
        ),
        create_blur_primary_bind_group(
            &device,
            &blur_primary_bind_group_layout,
            &basic_sampler,
            &target.ping_pong_buffers[0],
        ),
        create_blur_primary_bind_group(
            &device,
            &blur_primary_bind_group_layout,
            &basic_sampler,
            &target.ping_pong_buffers[1],
        ),
    ];

    let mut forward_primary_bind_group = create_forward_primary_bind_group(
        &device,
        &forward_primary_bind_group_layout,
        &basic_sampler,
        &target.hdr_buffer,
        &target.ping_pong_buffers[1],
    );

    let mut collada = ColladaReader::default();
    let mut cube_mesh = StaticMaterialMesh::default();
    collada.read_into(
        &mut util::buf_open("res/models/frigate.dae")?,
        &mut cube_mesh,
    )?;

    let cube_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(cube_mesh.vertices()),
        usage: BufferUsage::VERTEX,
    });

    let cube_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(cube_mesh.indices()),
        usage: BufferUsage::INDEX,
    });

    let mut rng = rand::thread_rng();
    let mut cube_models = vec![StaticMaterialMeshModel::default(); 512];
    let mut cube_transforms = vec![Transform::default(); 512];

    for (model, transform) in cube_models.iter_mut().zip(cube_transforms.iter_mut()) {
        *transform = transform.concat(&Transform {
            position: (
                rng.gen_range(-24.0, 24.0),
                rng.gen_range(-24.0, 24.0),
                rng.gen_range(-24.0, 24.0),
            )
                .into(),
            rotation: Quaternion::from_angle_up(rng.gen_range(0.0, math::TAU))
                * Quaternion::from_angle_right(rng.gen_range(0.0, math::TAU))
                * Quaternion::from_angle_forward(rng.gen_range(0.0, math::TAU)),
            ..Transform::default()
        });

        let matrix = (&*transform).into();
        model.model = matrix;
        model.inverse_normal = matrix.inversed().transposed().narrow();
    }

    let mut bmp_reader = BitmapReader::default();
    let mut diffuse_bmp = Bitmap::default();
    bmp_reader.read_into(
        &mut util::buf_open("res/bitmaps/frigate/diffuse.dds")?,
        &mut diffuse_bmp,
    )?;

    let mut normal_bmp = Bitmap::default();
    bmp_reader.read_into(
        &mut util::buf_open("res/bitmaps/frigate/normal.dds")?,
        &mut normal_bmp,
    )?;

    let mut specular_bmp = Bitmap::default();
    bmp_reader.read_into(
        &mut util::buf_open("res/bitmaps/frigate/specular.dds")?,
        &mut specular_bmp,
    )?;

    let mut emissive_bmp = Bitmap::default();
    bmp_reader.read_into(
        &mut util::buf_open("res/bitmaps/frigate/emissive.dds")?,
        &mut emissive_bmp,
    )?;

    let mut tex_manager = TextureManager::new(&device, 1024, 64);
    tex_manager.load_texture(&queue, diffuse_bmp, normal_bmp, specular_bmp, emissive_bmp)?;

    let static_material_texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &static_material_texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&tex_manager.diffuse_maps.1),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&tex_manager.specular_maps.1),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&tex_manager.emmisive_maps.1),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(&tex_manager.normal_maps.1),
            },
        ],
    });

    let mut frame_rate_timer = Instant::now();
    let mut frame_rate = 0;
    let mut update_timer = Instant::now();
    let mut update_delta_time = 0.0;
    let update_rate = Duration::from_secs_f32(1.0 / 60.0);

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
                        projection_dirty = Some(Vector2::from((w, h)));
                    }
                }
                Event::KeyDown { keycode, .. } => match keycode {
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
            let mouse_delta = mouse_pos - center;
            camera_euler_angles = Vector2::new(
                math::normalize_angle(camera_euler_angles.x() + mouse_delta.x() * 0.002),
                math::normalize_angle(camera_euler_angles.y() + -mouse_delta.y() * 0.002),
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

            // for (model, transform) in cube_models.iter_mut().zip(cube_transforms.iter_mut()) {
            //     *transform = transform.concat(&Transform {
            //         position: (
            //             rng.gen_range(-0.05, 0.05),
            //             rng.gen_range(-0.05, 0.05),
            //             rng.gen_range(-0.05, 0.05),
            //         )
            //             .into(),
            //         rotation: Quaternion::from_angle_up(rng.gen_range(-0.05, 0.05))
            //             * Quaternion::from_angle_right(rng.gen_range(-0.05, 0.05))
            //             * Quaternion::from_angle_forward(rng.gen_range(-0.05, 0.05)),
            //         ..Transform::default()
            //     });
            //
            //     let matrix = (&*transform).into();
            //     model.model = matrix;
            //     model.inverse_normal = matrix.inversed().transposed().narrow();
            // }
        }

        if mouse_dirty || physics_dirty {
            let view_parts = compute_view(camera_euler_angles, camera_position);
            queue.write_buffer(&view_buffer, 0, view_parts.0.to_bytes());
            frustum.update_look_at(camera_position, view_parts.1, Vector3::up());
        }

        if let Some(size) = projection_dirty {
            projection.aspect_ratio = size.x() / size.y();
            queue.write_buffer(
                &projection_buffer,
                0,
                compute_projection(&projection).to_bytes(),
            );
            frustum.update_projection(&projection);

            // Rre-bind the new HDR buffer since the size changed!
            forward_primary_bind_group = create_forward_primary_bind_group(
                &device,
                &forward_primary_bind_group_layout,
                &basic_sampler,
                &target.hdr_buffer,
                &target.ping_pong_buffers[1],
            );
            // And the blur bind-groups!
            blur_primary_bind_groups = [
                create_blur_primary_bind_group(
                    &device,
                    &blur_primary_bind_group_layout,
                    &basic_sampler,
                    &target.bloom_buffer,
                ),
                create_blur_primary_bind_group(
                    &device,
                    &blur_primary_bind_group_layout,
                    &basic_sampler,
                    &target.ping_pong_buffers[0],
                ),
                create_blur_primary_bind_group(
                    &device,
                    &blur_primary_bind_group_layout,
                    &basic_sampler,
                    &target.ping_pong_buffers[1],
                ),
            ];
        }

        // Pass 1: Draw the scene to the HDR buffer and also output the brightest parts to the
        // bloom buffer.
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[
                    RenderPassColorAttachmentDescriptor {
                        attachment: &target.hdr_buffer,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    },
                    RenderPassColorAttachmentDescriptor {
                        attachment: &target.bloom_buffer,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &target.depth_buffer,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&static_material_pipeline);
            render_pass.set_bind_group(0, &static_material_primary_bind_group, &[]);
            render_pass.set_bind_group(1, &static_material_texture_bind_group, &[]);

            for cube_model in &cube_models {
                if !frustum.sphere_inside(cube_model.model[3].narrow(), 2.0) {
                    continue;
                }
                render_pass.set_push_constants(
                    ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    0,
                    cube_model.to_words(),
                );
                render_pass.set_vertex_buffer(0, cube_vertex_buffer.slice(..));
                render_pass.set_index_buffer(cube_index_buffer.slice(..));
                render_pass.draw_indexed(0..cube_mesh.indices().len() as u32, 0, 0..1);
            }
        }

        // Pass 2-N: Gaussian blur the bloom buffer
        // Bounces "back and forth" blurring the bloom buffer inside the ping-pong buffers
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &target.ping_pong_buffers[0],
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&blur_pipeline);
            render_pass.set_bind_group(0, &blur_primary_bind_groups[0], &[]);
            render_pass.set_push_constants(
                ShaderStage::FRAGMENT,
                0,
                GaussianBlur {
                    horizontal: 0,
                    weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
                }
                .to_words(),
            );
            render_pass.set_vertex_buffer(0, output_target_vertex_buffer.slice(..));
            render_pass.draw(0..OUTPUT_TARGET_VERTICES.len() as u32, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &target.ping_pong_buffers[1],
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&blur_pipeline);
            render_pass.set_bind_group(0, &blur_primary_bind_groups[1], &[]);
            render_pass.set_push_constants(
                ShaderStage::FRAGMENT,
                0,
                GaussianBlur {
                    horizontal: 1,
                    weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
                }
                .to_words(),
            );
            render_pass.set_vertex_buffer(0, output_target_vertex_buffer.slice(..));
            render_pass.draw(0..OUTPUT_TARGET_VERTICES.len() as u32, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &target.ping_pong_buffers[0],
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&blur_pipeline);
            render_pass.set_bind_group(0, &blur_primary_bind_groups[2], &[]);
            render_pass.set_push_constants(
                ShaderStage::FRAGMENT,
                0,
                GaussianBlur {
                    horizontal: 0,
                    weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
                }
                .to_words(),
            );
            render_pass.set_vertex_buffer(0, output_target_vertex_buffer.slice(..));
            render_pass.draw(0..OUTPUT_TARGET_VERTICES.len() as u32, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &target.ping_pong_buffers[1],
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&blur_pipeline);
            render_pass.set_bind_group(0, &blur_primary_bind_groups[1], &[]);
            render_pass.set_push_constants(
                ShaderStage::FRAGMENT,
                0,
                GaussianBlur {
                    horizontal: 1,
                    weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
                }
                .to_words(),
            );
            render_pass.set_vertex_buffer(0, output_target_vertex_buffer.slice(..));
            render_pass.draw(0..OUTPUT_TARGET_VERTICES.len() as u32, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &target.ping_pong_buffers[0],
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&blur_pipeline);
            render_pass.set_bind_group(0, &blur_primary_bind_groups[2], &[]);
            render_pass.set_push_constants(
                ShaderStage::FRAGMENT,
                0,
                GaussianBlur {
                    horizontal: 0,
                    weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
                }
                .to_words(),
            );
            render_pass.set_vertex_buffer(0, output_target_vertex_buffer.slice(..));
            render_pass.draw(0..OUTPUT_TARGET_VERTICES.len() as u32, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &target.ping_pong_buffers[1],
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&blur_pipeline);
            render_pass.set_bind_group(0, &blur_primary_bind_groups[1], &[]);
            render_pass.set_push_constants(
                ShaderStage::FRAGMENT,
                0,
                GaussianBlur {
                    horizontal: 1,
                    weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
                }
                .to_words(),
            );
            render_pass.set_vertex_buffer(0, output_target_vertex_buffer.slice(..));
            render_pass.draw(0..OUTPUT_TARGET_VERTICES.len() as u32, 0..1);
        }

        // The render buffers will automatically be swapped when this texture drops
        let current_frame = target.swap_chain.get_current_frame()?;

        // Final Pass: Merge the HDR and blur buffer
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &current_frame.output.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&forward_pipeline);
            render_pass.set_bind_group(0, &forward_primary_bind_group, &[]);
            render_pass.set_push_constants(ShaderStage::FRAGMENT, 0, Exposure(0.8).to_words());
            render_pass.set_vertex_buffer(0, output_target_vertex_buffer.slice(..));
            render_pass.draw(0..OUTPUT_TARGET_VERTICES.len() as u32, 0..1);
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

fn main() -> Result<(), BoxedError> {
    assert!(mem::size_of::<StaticMaterialMeshModel>() <= MAX_PUSH_CONSTANT_SIZE);

    env_logger::builder()
        .filter_level(LevelFilter::Error)
        .filter_module("dth", LevelFilter::Debug)
        .init();

    match main_real() {
        Err(err) => {
            log::error!("{:?}", err);
            Err(err)
        }
        _ => Ok(()),
    }
}
