use gloo::utils::document;
use std::borrow::Cow;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wgpu::InstanceDescriptor;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

async fn run() {
    let canvas: web_sys::Element = document().get_element_by_id("render").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;

    log::debug!("width : {}, height : {}", width, height);

    let instance: wgpu::Instance = wgpu::Instance::new(InstanceDescriptor::default());
    let surface: wgpu::Surface = instance
        .create_surface_from_canvas(canvas)
        .expect("Failed to create surface from canvas");

    let adapter: wgpu::Adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout: wgpu::PipelineLayout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

    let swapchain_capabilities: wgpu::SurfaceCapabilities = surface.get_capabilities(&adapter);
    let swapchain_format: wgpu::TextureFormat = swapchain_capabilities.formats[0];

    let render_pipeline: wgpu::RenderPipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: width,
        height: height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    // ===============================================================

    let frame: wgpu::SurfaceTexture = surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");
    let view: wgpu::TextureView = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut rpass: wgpu::RenderPass<'_> =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        rpass.set_pipeline(&render_pipeline);
        rpass.draw(0..3, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    frame.present();

    log::debug!("Run End OK");
}

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Begin OK");

    spawn_local(run());
}
