use crate::engine::define;
use crate::rendering::common;

use wasm_bindgen::JsCast;
use wgpu::util::DeviceExt;

pub struct WebGPUContext<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub shader: wgpu::ShaderModule,
    pub swapchain_format: wgpu::TextureFormat,
    pub depth_texture: wgpu::Texture,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buf: wgpu::Buffer,
    pub render_pipeline: wgpu::RenderPipeline,
}

pub async fn init<'a>() -> WebGPUContext<'a> {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .expect("Failed to get canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into()
        .expect("Failed to dynamic cast canvas element");

    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;

    // webgpu context

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

    // buffers

    let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
            "../shader/unlit.wgsl" // TODO: Convert to constant variable
        ))),
    });

    let vertex_size: usize = std::mem::size_of::<common::Vertex>();
    let (vertex_data, index_data) = common::create_cube();

    let vertex_buf: wgpu::Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertex_data),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buf: wgpu::Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&index_data),
        usage: wgpu::BufferUsages::INDEX,
    });

    // bindings

    let mvp_total = common::create_mvp(width as f32 / height as f32);
    let mvp_ref: &[f32; 16] = mvp_total.as_ref();
    let uniform_buf: wgpu::Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(mvp_ref),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let texture = device.create_texture_with_data(
        &queue,
        &wgpu::TextureDescriptor {
            label: (None),
            size: wgpu::Extent3d {
                width: 2,
                height: 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: (1),
            sample_count: (1),
            dimension: (wgpu::TextureDimension::D2),
            format: (wgpu::TextureFormat::Rgba8Unorm),
            usage: (wgpu::TextureUsages::TEXTURE_BINDING),
            view_formats: (&[]),
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &[
            255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255, 255, 255, 255, 255,
        ],
    );
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group_layout: wgpu::BindGroupLayout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("Bind group 0"),
    });

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
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    // pipeline

    let pipeline_layout: wgpu::PipelineLayout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                format: wgpu::VertexFormat::Float32x2,
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 1,
            },
        ],
    }];

    let render_pipeline: wgpu::RenderPipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
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

    // Return packed values

    let context: WebGPUContext<'_> = WebGPUContext {
        surface,
        device,
        queue,
        shader,
        swapchain_format,
        depth_texture,
        vertex_buf,
        index_buf,
        index_count,
        bind_group,
        bind_group_layout,
        uniform_buf,
        render_pipeline,
    };
    return context;
}

pub fn override_context(context: &mut WebGPUContext, model: &tobj::Model) {
    let vertex_data: Vec<common::Vertex> = common::create_vertices_from_obj(model, true);
    let index_data: Vec<u32> = model.mesh.indices.clone();
    let vertex_size: usize = std::mem::size_of::<common::Vertex>();

    let vertex_buf: wgpu::Buffer =
        context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

    let index_buf: wgpu::Buffer =
        context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

    // ---------------------------------------------------------------------------

    let pipeline_layout: wgpu::PipelineLayout =
        context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&context.bind_group_layout],
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
                format: wgpu::VertexFormat::Float32x2,
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 1,
            },
        ],
    }];

    let render_pipeline: wgpu::RenderPipeline =
        context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &context.shader,
                    entry_point: Some(define::VS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &context.shader,
                    entry_point: Some(define::FS_ENTRY_POINT),
                    compilation_options: Default::default(),
                    targets: &[Some(context.swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
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

    // Update context
    context.vertex_buf = vertex_buf;
    context.index_buf = index_buf;
    context.render_pipeline = render_pipeline;
    context.index_count = index_count;
}

pub fn render(context: &WebGPUContext) {
    let frame: wgpu::SurfaceTexture = context
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view: wgpu::TextureView = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let depth_texture_view: wgpu::TextureView =
        context
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("depth texture view"),
                format: Some(wgpu::TextureFormat::Depth24Plus),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_array_layer: 0,
                array_layer_count: Some(1),
                base_mip_level: 0,
                mip_level_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
            });

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut rpass: wgpu::RenderPass<'_> =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.8,
                            g: 0.8,
                            b: 0.8,
                            a: 1.0,
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
        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(&context.render_pipeline);
        rpass.set_bind_group(0, &context.bind_group, &[]);
        rpass.set_index_buffer(context.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, context.vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..context.index_count as u32, 0, 0..1);
    }

    context.queue.submit(Some(encoder.finish()));
    frame.present();
}
