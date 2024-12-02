use crate::engine::define;
use crate::rendering::common;

use wasm_bindgen::JsCast;
use wgpu::util::DeviceExt;

pub const WGPU_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

pub struct WebGPUInterface<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swapchain_format: wgpu::TextureFormat,
    pub depth_texture: wgpu::Texture,
}

pub struct WebGPURenderResource {
    pub shader: wgpu::ShaderModule,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buf: wgpu::Buffer,
    pub render_pipeline: wgpu::RenderPipeline,
}

// --------------------------------------------------------------------------------

pub async fn init_webgpu<'a>() -> WebGPUInterface<'a> {
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
        format: WGPU_DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
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

// --------------------------------------------------------------------------------

pub fn init_webgpu_color_shader(interface: &WebGPUInterface) -> WebGPURenderResource {
    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .expect("Failed to get canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into()
        .expect("Failed to dynamic cast canvas element");

    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;

    let shader: wgpu::ShaderModule =
        interface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../shader/color.wgsl" // TODO: Convert to constant variable
                ))),
            });

    let vertex_size: usize = std::mem::size_of::<common::Vertex>();
    let (vertex_data, index_data) = common::create_cube();

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

    let mvp_total = common::create_mvp(width as f32 / height as f32);
    let mvp_ref: &[f32; 16] = mvp_total.as_ref();
    let uniform_buf: wgpu::Buffer =
        interface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(mvp_ref),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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

    let render_resource: WebGPURenderResource = WebGPURenderResource {
        shader,
        vertex_buf,
        index_buf,
        index_count,
        bind_group,
        bind_group_layout,
        uniform_buf,
        render_pipeline,
    };

    return render_resource;
}

// --------------------------------------------------------------------------------

pub fn render_main(interface: &WebGPUInterface, resource: &WebGPURenderResource) {
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
                format: Some(wgpu::TextureFormat::Depth24Plus),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_array_layer: 0,
                array_layer_count: Some(1),
                base_mip_level: 0,
                mip_level_count: Some(1),
                dimension: Some(wgpu::TextureViewDimension::D2),
            });

    let mut encoder = interface
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
        rpass.set_pipeline(&resource.render_pipeline);
        rpass.set_bind_group(0, &resource.bind_group, &[]);
        rpass.set_index_buffer(resource.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, resource.vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..resource.index_count as u32, 0, 0..1);
    }

    interface.queue.submit(Some(encoder.finish()));
    frame.present();
}