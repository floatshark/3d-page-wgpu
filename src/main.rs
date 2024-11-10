use bytemuck::{Pod, Zeroable};
use glam::{self, Vec4Swizzles};
use gloo::utils::document;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wgpu::util::DeviceExt;

// Small Size Allocator for Optimization
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Constant Variables
const UPDATE_FPS: u32 = 60;
const CANVAS_ELEMENT_ID: &str = "canvas";
const VS_ENTRY_POINT: &str = "vs_main";
const FS_ENTRY_POINT: &str = "fs_main";
macro_rules! SHADER_FILE_NAME {
    () => {
        "shader.wgsl"
    };
}

struct RenderContext<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
}

#[derive(Clone, Copy)]
struct ViewRecord {
    movement_x: i32,
    movement_y: i32,
    on_click: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
}

fn vertex(pos: [i8; 3]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1]),
        vertex([1, -1, 1]),
        vertex([1, 1, 1]),
        vertex([-1, 1, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1]),
        vertex([1, 1, -1]),
        vertex([1, -1, -1]),
        vertex([-1, -1, -1]),
        // right (1, 0, 0)
        vertex([1, -1, -1]),
        vertex([1, 1, -1]),
        vertex([1, 1, 1]),
        vertex([1, -1, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1]),
        vertex([-1, 1, 1]),
        vertex([-1, 1, -1]),
        vertex([-1, -1, -1]),
        // front (0, 1, 0)
        vertex([1, 1, -1]),
        vertex([-1, 1, -1]),
        vertex([-1, 1, 1]),
        vertex([1, 1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1]),
        vertex([-1, -1, 1]),
        vertex([-1, -1, -1]),
        vertex([1, -1, -1]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_mvp(aspect_ratio: f32) -> glam::Mat4 {
    let projection =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(1.5f32, -5.0, 3.0),
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    projection * view
}

async fn init<'a>() -> RenderContext<'a> {
    let canvas: web_sys::Element = document().get_element_by_id(CANVAS_ELEMENT_ID).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;

    // -----

    let instance: wgpu::Instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface_target = wgpu::SurfaceTarget::Canvas(canvas);
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
        .unwrap();

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
        .unwrap();

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

    // vertex ~~~~~~

    let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
            SHADER_FILE_NAME!()
        ))),
    });

    let vertex_size = std::mem::size_of::<Vertex>();
    let (vertex_data, index_data) = create_vertices();

    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertex_data),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&index_data),
        usage: wgpu::BufferUsages::INDEX,
    });

    // binding ~~~~~~~

    let mvp_total = create_mvp(width as f32 / height as f32);
    let mvp_ref: &[f32; 16] = mvp_total.as_ref();
    let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(mvp_ref),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buf.as_entire_binding(),
        }],
        label: None,
    });

    // pipeline -----------------

    let pipeline_layout: wgpu::PipelineLayout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

    let vertex_buffers = [wgpu::VertexBufferLayout {
        array_stride: vertex_size as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        }],
    }];

    let render_pipeline: wgpu::RenderPipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: VS_ENTRY_POINT,
                compilation_options: Default::default(),
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: FS_ENTRY_POINT,
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

    let index_count: u32 = index_data.len() as u32;

    let context = RenderContext {
        surface,
        device,
        queue,
        vertex_buf,
        index_buf,
        index_count,
        bind_group,
        uniform_buf,
        render_pipeline,
    };
    return context;
}

fn add_event_listener(view_record: &'static mut ViewRecord) {
    let canvas: web_sys::Element = document().get_element_by_id(CANVAS_ELEMENT_ID).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let mouse_move_closure = Closure::<dyn FnMut(_)>::new(|event: web_sys::MouseEvent| {
        view_record.movement_x = event.movement_x();
        view_record.movement_y = event.movement_y();
        view_record.on_click = event.buttons() == 1;
    });
    canvas
        .add_event_listener_with_callback("mousemove", mouse_move_closure.as_ref().unchecked_ref())
        .unwrap();
    mouse_move_closure.forget();
}

fn update(render_context: &RenderContext, view_record: &ViewRecord) {
    static mut eye: glam::Vec3 = glam::Vec3::new(1.5f32, -5.0, 3.0);

    let enable_mouse_control = view_record.on_click;
    if enable_mouse_control {
        let canvas: web_sys::Element = document().get_element_by_id(CANVAS_ELEMENT_ID).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

        let width: u32 = canvas.client_width() as u32;
        let height: u32 = canvas.client_height() as u32;
        let aspect_ratio = width as f32 / height as f32;

        unsafe{
            let rotate_x_quat = glam::Quat::from_rotation_z(-1.0 * view_record.movement_x as f32 * 0.01);
            eye = rotate_x_quat.mul_vec3(eye);
        }

        unsafe{
            let y_axis = glam::vec3(eye.x, eye.y, eye.z).cross(glam::vec3(0.0, 0.0, 1.0)).normalize();
            let rotate_y_quat = glam::Quat::from_axis_angle(y_axis, view_record.movement_y as f32 * 0.01);
            eye = rotate_y_quat.mul_vec3(eye);
        }

        let mut eye_pos = glam::Vec3::new(1.5f32, -5.0, 3.0);
        unsafe{eye_pos = glam::vec3(eye.x, eye.y, eye.z);}

        let view = glam::Mat4::look_at_rh(eye_pos, glam::Vec3::ZERO, glam::Vec3::Z);

        let projection =
            glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);

        let mx_total = projection * view;
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        render_context.queue.write_buffer(
            &render_context.uniform_buf,
            0,
            bytemuck::cast_slice(mx_ref),
        );
    }
}

fn render(context: &RenderContext) {
    let frame: wgpu::SurfaceTexture = context
        .surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    let view: wgpu::TextureView = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(&context.render_pipeline);
        rpass.set_bind_group(0, &context.bind_group, &[]);
        rpass.set_index_buffer(context.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, context.vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..context.index_count as u32, 0, 0..1);
    }

    context.queue.submit(Some(encoder.finish()));
    frame.present();
}

#[wasm_bindgen(main)]
pub async fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    let render_context: RenderContext = init().await;

    static mut view_record: ViewRecord = ViewRecord {
        movement_x: 0,
        movement_y: 0,
        on_click: false,
    };
    unsafe {
        add_event_listener(&mut view_record);
    }

    game_loop::game_loop(
        unsafe { (render_context, &view_record) },
        UPDATE_FPS,
        0.1,
        |g| {
            update(&g.game.0, &g.game.1);
        },
        |g| {
            render(&g.game.0);
        },
    );
}
