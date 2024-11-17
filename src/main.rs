mod frontend;
mod rendering;

use wasm_bindgen::JsCast;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const UPDATE_FPS: u32 = 60;
const CANVAS_ELEMENT_ID: &str = "canvas";

#[wasm_bindgen::prelude::wasm_bindgen(main)]
pub async fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    let webgpu_context: rendering::webgpu::WebGPUContext = rendering::webgpu::init().await;

    let mouse_event_js: std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>> =
        std::rc::Rc::new(std::cell::Cell::new(
            frontend::controls::MouseEventResponseJs {
                movement_x: 0,
                movement_y: 0,
                on_click: false,
            },
        ));
    frontend::controls::add_event_listener_control(&mouse_event_js);

    let view: std::rc::Rc<std::cell::Cell<View>> = std::rc::Rc::new(std::cell::Cell::new(View {
        eye: glam::Vec3::new(1.5f32, -5.0, 3.0),
    }));

    let view_clone = view.clone();

    game_loop::game_loop(
        (webgpu_context, mouse_event_js, view_clone),
        UPDATE_FPS,
        0.1,
        |g: &mut game_loop::GameLoop<
            (
                rendering::webgpu::WebGPUContext<'_>,
                std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>>,
                std::rc::Rc<std::cell::Cell<View>>,
            ),
            game_loop::Time,
            (),
        >| {
            update(&g.game.0, &g.game.1, &g.game.2);
        },
        |g: &mut game_loop::GameLoop<
            (
                rendering::webgpu::WebGPUContext<'_>,
                std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>>,
                std::rc::Rc<std::cell::Cell<View>>,
            ),
            game_loop::Time,
            (),
        >| {
            render(&g.game.0);
        },
    );
}

#[derive(Clone, Copy)]
struct View {
    eye: glam::Vec3,
}

fn update(
    render_context: &rendering::webgpu::WebGPUContext,
    mouse_event_js: &std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>>,
    view: &std::rc::Rc<std::cell::Cell<View>>,
) {
    let mut eye: glam::Vec3 = view.get().eye;

    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;
    let aspect_ratio: f32 = width as f32 / height as f32;

    let enable_mouse_control = mouse_event_js.get().on_click;
    if enable_mouse_control {
        let rotate_x_quat =
            glam::Quat::from_rotation_z(-1.0 * mouse_event_js.get().movement_x as f32 * 0.01);
        eye = rotate_x_quat.mul_vec3(eye);

        let y_axis = glam::vec3(eye.x, eye.y, eye.z)
            .cross(glam::vec3(0.0, 0.0, 1.0))
            .normalize();
        let rotate_y_quat =
            glam::Quat::from_axis_angle(y_axis, mouse_event_js.get().movement_y as f32 * 0.01);
        eye = rotate_y_quat.mul_vec3(eye);
    }

    let view_matrix: glam::Mat4 = glam::Mat4::look_at_rh(eye, glam::Vec3::ZERO, glam::Vec3::Z);
    let projection_matrix: glam::Mat4 =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);

    let mx_total: glam::Mat4 = projection_matrix * view_matrix;
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    render_context
        .queue
        .write_buffer(&render_context.uniform_buf, 0, bytemuck::cast_slice(mx_ref));

    let view_temp: View = View { eye: eye };
    view.set(view_temp);
}

fn render(context: &rendering::webgpu::WebGPUContext) {
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
