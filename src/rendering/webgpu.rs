use crate::engine::{self, define};
use crate::rendering::common;

use wasm_bindgen::JsCast;
use wgpu::util::DeviceExt;
use wgpu::TextureViewDescriptor;

const WEBGPU_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
const WEBGPU_FRONT_FACE: wgpu::FrontFace = wgpu::FrontFace::Ccw;
const WEBGPU_CULL_MODE: wgpu::Face = wgpu::Face::Back;

pub struct WebGPUInterface<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swapchain_format: wgpu::TextureFormat,
    pub depth_texture: wgpu::Texture,
}

pub struct WebGPURenderResource {
    pub _shader: wgpu::ShaderModule,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
    pub bind_group: wgpu::BindGroup,
    pub _bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buf: wgpu::Buffer,
    pub render_pipeline: wgpu::RenderPipeline,
}

pub struct WebGPUDifferedGBuffer {
    gbuffer_position_texture: wgpu::Texture,
    gbuffer_normal_texture: wgpu::Texture,
}

pub struct WebGPUDifferedResource {
    pub _shader: wgpu::ShaderModule,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub uniform_buf: wgpu::Buffer,
    pub render_pipeline: wgpu::RenderPipeline,
    pub debug_pipeline: wgpu::RenderPipeline,
}

// Webgpu contexts --------------------------------------------------------------------------------

pub async fn init_interface<'a>() -> WebGPUInterface<'a> {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .expect("Failed to get canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into()
        .expect("Failed to dynamic cast canvas element");

    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;

    // Initialize webgpu

    let instance: wgpu::Instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface_target: wgpu::SurfaceTarget<'_> = wgpu::SurfaceTarget::Canvas(canvas);
    let surface: wgpu::Surface = instance
        .create_surface(surface_target)
        .expect("Failed to create surface from canvas");

    let adapter: wgpu::Adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to request adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        )
        .await
        .expect("Failed to request device");

    let swapchain_capabilities: wgpu::SurfaceCapabilities = surface.get_capabilities(&adapter);
    let swapchain_format: wgpu::TextureFormat = swapchain_capabilities.formats[0];

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: width,
        height: height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let depth_texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: WEBGPU_DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    // Return webgpu resource

    let resource: WebGPUInterface<'_> = WebGPUInterface {
        surface,
        device,
        queue,
        swapchain_format,
        depth_texture,
    };

    return resource;
}

pub fn init_differed_gbuffer(interface: &WebGPUInterface) -> WebGPUDifferedGBuffer {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .expect("Failed to get canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into()
        .expect("Failed to dynamic cast canvas element");

    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;

    // Gbuffers

    let gbuffer_position_texture: wgpu::Texture =
        interface.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("position texture"),
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

    let gbuffer_normal_texture: wgpu::Texture =
        interface.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("normal texture"),
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

    WebGPUDifferedGBuffer {
        gbuffer_position_texture,
        gbuffer_normal_texture,
    }
}

// Differed rendering -----------------------------------------------------------------------------
struct WriteGBuffersUniform {
    _transform_matrix: [f32; 16],
}

pub fn init_differed_gbuffers_shader(
    interface: &WebGPUInterface,
    mesh: &common::Mesh,
) -> WebGPURenderResource {
    let shader: wgpu::ShaderModule =
        interface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shader/differed_write_gbuffers.wgsl"
                ))),
            });

    let vertex_size: usize = std::mem::size_of::<common::Vertex>();
    let vertex_data: &Vec<common::Vertex> = &mesh.vertices;
    let index_data: &Vec<u32> = &mesh.indices;

    let vertex_buf: wgpu::Buffer =
        interface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

    let index_buf: wgpu::Buffer =
        interface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

    // bindings

    let uniform_size: u64 = std::mem::size_of::<WriteGBuffersUniform>() as u64;
    let uniform_buf: wgpu::Buffer = interface.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: uniform_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout: wgpu::BindGroupLayout =
        interface
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                }],
            });

    let bind_group: wgpu::BindGroup =
        interface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
                label: Some("Bind group 0"),
            });

    // pipeline

    let pipeline_layout: wgpu::PipelineLayout =
        interface
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

    let vertex_buffers: [wgpu::VertexBufferLayout<'_>; 1] = [wgpu::VertexBufferLayout {
        array_stride: vertex_size as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: std::mem::size_of::<[f32; 9]>() as u64,
                shader_location: 1,
            },
        ],
    }];

    let render_pipeline: wgpu::RenderPipeline =
        interface
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(define::VS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(define::FS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba16Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        }),
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba16Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        }),
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(WEBGPU_CULL_MODE),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: WEBGPU_DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

    let index_count: u32 = index_data.len() as u32;

    let render_resource: WebGPURenderResource = WebGPURenderResource {
        _shader: shader,
        vertex_buf,
        index_buf,
        index_count,
        bind_group,
        _bind_group_layout: bind_group_layout,
        uniform_buf,
        render_pipeline,
    };

    return render_resource;
}

pub fn upadte_differed_gbuffers_buffer(
    scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>,
    interface: &WebGPUInterface,
    resource: &WebGPURenderResource,
) {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;
    let aspect_ratio: f32 = width as f32 / height as f32;

    let eye: glam::Vec3 = scene.get().eye_location;
    let direction: glam::Vec3 = scene.get().eye_direction;

    // Create matrices and write buffer
    let view_matrix = glam::Mat4::look_to_rh(eye, direction, glam::Vec3::Z);
    let projection_matrix: glam::Mat4 =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.01, 100.0);
    let mx_total: glam::Mat4 = projection_matrix * view_matrix;
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    interface
        .queue
        .write_buffer(&resource.uniform_buf, 0, bytemuck::cast_slice(mx_ref));
}

struct DifferedUniform {
    _directional_light: [f32; 4],
    _ambient_light: [f32; 4],
    _inverse_matrix: [f32; 16],
    _debug: DifferedDebugUniform,
}
struct DifferedDebugUniform {
    _buffer_type: f32,
    _padding: [f32; 3],
}

pub fn init_differed_shading(
    interface: &WebGPUInterface,
    gbuffer: &WebGPUDifferedGBuffer,
) -> WebGPUDifferedResource {
    let shader: wgpu::ShaderModule =
        interface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shader/differed.wgsl"
                ))),
            });

    // bindings

    let gbuffer_bind_group_layout: wgpu::BindGroupLayout = interface
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

    let gbuffer_bind_group: wgpu::BindGroup =
        interface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &gbuffer_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &gbuffer
                                .gbuffer_position_texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &gbuffer
                                .gbuffer_normal_texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &interface
                                .depth_texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                ],
                label: Some("Bind group 0"),
            });

    let uniform_size: u64 = std::mem::size_of::<DifferedUniform>() as u64;
    let uniform_buf: wgpu::Buffer = interface.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Differed uniform buffer"),
        size: uniform_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_bind_group_layout: wgpu::BindGroupLayout = interface
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let uniform_bind_group: wgpu::BindGroup =
        interface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
                label: Some("Bind group 1"),
            });

    // pipeline

    let pipeline_layout: wgpu::PipelineLayout =
        interface
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&gbuffer_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

    let render_pipeline: wgpu::RenderPipeline =
        interface
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(define::VS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(engine::define::FS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    targets: &[Some(interface.swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(WEBGPU_CULL_MODE),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

    let debug_pipeline: wgpu::RenderPipeline =
        interface
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(define::VS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_debug_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(interface.swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(WEBGPU_CULL_MODE),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

    let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();
    bind_groups.push(gbuffer_bind_group);
    bind_groups.push(uniform_bind_group);

    let resource: WebGPUDifferedResource = WebGPUDifferedResource {
        _shader: shader,
        bind_groups,
        uniform_buf,
        render_pipeline,
        debug_pipeline,
    };

    return resource;
}

pub fn update_differed_buffer(
    scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>,
    interface: &WebGPUInterface,
    resource: &WebGPUDifferedResource,
) {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;
    let aspect_ratio: f32 = width as f32 / height as f32;

    let eye: glam::Vec3 = scene.get().eye_location;
    let direction: glam::Vec3 = scene.get().eye_direction;

    // Create matrices and write buffer
    let view_matrix = glam::Mat4::look_to_rh(eye, direction, glam::Vec3::Z);
    let projection_matrix: glam::Mat4 =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.01, 100.0);
    let transform_matrix: glam::Mat4 = projection_matrix * view_matrix;

    let directional: [f32; 3] = scene.get().directional_light_angle;
    let ambient: [f32; 4] = scene.get().ambient_light_color;
    let inverse_projection: glam::Mat4 = transform_matrix.inverse();

    let mut uniform_total: Vec<f32> = Vec::new();
    uniform_total.extend_from_slice(&directional);
    uniform_total.extend_from_slice(&[0.0]); // Padding!
    uniform_total.extend_from_slice(&ambient);
    uniform_total.extend_from_slice(&inverse_projection.to_cols_array().to_vec());
    uniform_total.extend_from_slice(&[scene.get().differed_debug_type as f32, 0.0, 0.0, 0.0]);

    let uniform_ref: &[f32] = uniform_total.as_ref();
    interface
        .queue
        .write_buffer(&resource.uniform_buf, 0, bytemuck::cast_slice(uniform_ref));
}

// Forward vertex color shader --------------------------------------------------------------------

#[allow(dead_code)]
pub fn init_color_shader(interface: &WebGPUInterface, mesh: &common::Mesh) -> WebGPURenderResource {
    let shader: wgpu::ShaderModule =
        interface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shader/color.wgsl"
                ))),
            });

    let vertex_size: usize = std::mem::size_of::<common::Vertex>();
    let vertex_data: &Vec<common::Vertex> = &mesh.vertices;
    let index_data: &Vec<u32> = &mesh.indices;

    let vertex_buf: wgpu::Buffer =
        interface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

    let index_buf: wgpu::Buffer =
        interface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

    // bindings

    let uniform_size: u64 = 4 * (16);
    let uniform_buf: wgpu::Buffer = interface.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: uniform_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout: wgpu::BindGroupLayout =
        interface
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                }],
            });

    let bind_group: wgpu::BindGroup =
        interface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
                label: Some("Bind group 0"),
            });

    // pipeline

    let pipeline_layout: wgpu::PipelineLayout =
        interface
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

    let vertex_buffers: [wgpu::VertexBufferLayout<'_>; 1] = [wgpu::VertexBufferLayout {
        array_stride: vertex_size as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: (std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                shader_location: 1,
            },
        ],
    }];

    let render_pipeline: wgpu::RenderPipeline =
        interface
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(define::VS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(define::FS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    targets: &[Some(interface.swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(WEBGPU_CULL_MODE),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

    let index_count: u32 = index_data.len() as u32;

    let render_resource: WebGPURenderResource = WebGPURenderResource {
        _shader: shader,
        vertex_buf,
        index_buf,
        index_count,
        bind_group,
        _bind_group_layout: bind_group_layout,
        uniform_buf,
        render_pipeline,
    };

    return render_resource;
}

#[allow(dead_code)]
pub fn update_color_buffer(
    scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>,
    interface: &WebGPUInterface,
    resource: &WebGPURenderResource,
) {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;
    let aspect_ratio: f32 = width as f32 / height as f32;

    let eye: glam::Vec3 = scene.get().eye_location;
    let direction: glam::Vec3 = scene.get().eye_direction;

    // Create matrices and write buffer
    let view_matrix = glam::Mat4::look_to_rh(eye, direction, glam::Vec3::Z);
    let projection_matrix: glam::Mat4 =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.01, 100.0);
    let mx_total: glam::Mat4 = projection_matrix * view_matrix;
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    interface
        .queue
        .write_buffer(&resource.uniform_buf, 0, bytemuck::cast_slice(mx_ref));
}

// Forward phong shader----------------------------------------------------------------------------

#[allow(dead_code)]
pub fn init_phong_shader(interface: &WebGPUInterface, mesh: &common::Mesh) -> WebGPURenderResource {
    struct PhongUniform {
        transform_matrix: [f32; 16],
        directional_light: [f32; 4],
        ambient_light: [f32; 4],
        inverse_matrix: [f32; 16],
    }

    let shader: wgpu::ShaderModule =
        interface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shader/phong.wgsl"
                ))),
            });

    let vertex_size: usize = std::mem::size_of::<common::Vertex>();
    let vertex_data: &Vec<common::Vertex> = &mesh.vertices;
    let index_data: &Vec<u32> = &mesh.indices;

    let vertex_buf: wgpu::Buffer =
        interface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

    let index_buf: wgpu::Buffer =
        interface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

    let uniform_size: u64 = std::mem::size_of::<PhongUniform>() as u64;
    let uniform_buf: wgpu::Buffer = interface.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: uniform_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout: wgpu::BindGroupLayout =
        interface
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(uniform_size),
                    },
                    count: None,
                }],
            });

    let bind_group: wgpu::BindGroup =
        interface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
                label: Some("Bind group 0"),
            });

    // pipeline

    let pipeline_layout: wgpu::PipelineLayout =
        interface
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

    let vertex_buffers: [wgpu::VertexBufferLayout<'_>; 1] = [wgpu::VertexBufferLayout {
        array_stride: vertex_size as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: std::mem::size_of::<[f32; 9]>() as u64,
                shader_location: 1,
            },
        ],
    }];

    let render_pipeline: wgpu::RenderPipeline =
        interface
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(define::VS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(define::FS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    targets: &[Some(interface.swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: WEBGPU_FRONT_FACE,
                    cull_mode: Some(WEBGPU_CULL_MODE),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: WEBGPU_DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

    let index_count: u32 = index_data.len() as u32;

    let render_resource: WebGPURenderResource = WebGPURenderResource {
        _shader: shader,
        vertex_buf,
        index_buf,
        index_count,
        bind_group,
        _bind_group_layout: bind_group_layout,
        uniform_buf,
        render_pipeline,
    };

    return render_resource;
}

#[allow(dead_code)]
pub fn update_phong_buffer(
    scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>,
    interface: &WebGPUInterface,
    resource: &WebGPURenderResource,
) {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;
    let aspect_ratio: f32 = width as f32 / height as f32;

    let eye: glam::Vec3 = scene.get().eye_location;
    let direction: glam::Vec3 = scene.get().eye_direction;

    // Create matrices and write buffer
    let view_matrix = glam::Mat4::look_to_rh(eye, direction, glam::Vec3::Z);
    let projection_matrix: glam::Mat4 =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.01, 100.0);
    let transform_matrix: glam::Mat4 = projection_matrix * view_matrix;

    let directional: [f32; 3] = scene.get().directional_light_angle;
    let ambient: [f32; 4] = scene.get().ambient_light_color;
    let inverse_projection: glam::Mat4 = transform_matrix.inverse();

    let mut uniform_total: Vec<f32> = transform_matrix.to_cols_array().to_vec();
    uniform_total.extend_from_slice(&directional);
    uniform_total.extend_from_slice(&[0.0]); // Padding!
    uniform_total.extend_from_slice(&ambient);
    uniform_total.extend_from_slice(&inverse_projection.to_cols_array().to_vec());

    let uniform_ref: &[f32] = uniform_total.as_ref();
    interface
        .queue
        .write_buffer(&resource.uniform_buf, 0, bytemuck::cast_slice(uniform_ref));
}

// Render functions--------------------------------------------------------------------------------

pub fn render_forward_main(
    interface: &WebGPUInterface,
    scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>,
    resources: &Vec<WebGPURenderResource>,
) {
    let frame: wgpu::SurfaceTexture = interface
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view: wgpu::TextureView = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let depth_texture_view: wgpu::TextureView =
        interface
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("depth texture view"),
                format: Some(WEBGPU_DEPTH_FORMAT),
                aspect: wgpu::TextureAspect::All,
                base_array_layer: 0,
                array_layer_count: Some(1),
                base_mip_level: 0,
                mip_level_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
            });

    let mut encoder: wgpu::CommandEncoder =
        interface
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Forward render encoder"),
            });

    // Forward main path
    {
        let mut rpass: wgpu::RenderPass<'_> =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Forward render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: scene.get().background_color[0] as f64,
                            g: scene.get().background_color[1] as f64,
                            b: scene.get().background_color[2] as f64,
                            a: scene.get().background_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        for resource in resources {
            rpass.set_pipeline(&resource.render_pipeline);
            rpass.set_bind_group(0, &resource.bind_group, &[]);
            rpass.set_index_buffer(resource.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.set_vertex_buffer(0, resource.vertex_buf.slice(..));
            rpass.draw_indexed(0..resource.index_count, 0, 0..1);
        }
    }

    interface.queue.submit(Some(encoder.finish()));
    frame.present();
}

pub fn render_differed_main(
    interface: &WebGPUInterface,
    scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>,
    gbuffer: &WebGPUDifferedGBuffer,
    resources: &Vec<WebGPURenderResource>,
    differed_resource: &WebGPUDifferedResource,
) {
    let frame: wgpu::SurfaceTexture = interface
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view: wgpu::TextureView = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder: wgpu::CommandEncoder =
        interface
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Differed render encoder"),
            });

    // gbuffer pass
    {
        let mut gbuffer_pass: wgpu::RenderPass<'_> =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Differed gbuffer pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &gbuffer
                            .gbuffer_position_texture
                            .create_view(&TextureViewDescriptor::default()),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &gbuffer
                            .gbuffer_normal_texture
                            .create_view(&TextureViewDescriptor::default()),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &interface
                        .depth_texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        for resource in resources {
            gbuffer_pass.set_pipeline(&resource.render_pipeline);
            gbuffer_pass.set_bind_group(0, &resource.bind_group, &[]);
            gbuffer_pass.set_index_buffer(resource.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            gbuffer_pass.set_vertex_buffer(0, resource.vertex_buf.slice(..));
            gbuffer_pass.draw_indexed(0..resource.index_count, 0, 0..1);
        }
    }

    // differed pass
    {
        let mut differed_pass: wgpu::RenderPass<'_> =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Differed render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: scene.get().background_color[0] as f64,
                            g: scene.get().background_color[1] as f64,
                            b: scene.get().background_color[2] as f64,
                            a: scene.get().background_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        if scene.get().differed_debug_type == 0 {
            differed_pass.set_pipeline(&differed_resource.render_pipeline);
        } else {
            differed_pass.set_pipeline(&differed_resource.debug_pipeline);
        }

        differed_pass.set_bind_group(0, &differed_resource.bind_groups[0], &[]);
        differed_pass.set_bind_group(1, &differed_resource.bind_groups[1], &[]);
        differed_pass.draw(0..6, 0..1);
    }

    interface.queue.submit(Some(encoder.finish()));
    frame.present();
}
