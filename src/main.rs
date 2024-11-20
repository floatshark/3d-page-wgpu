mod frontend;
mod rendering;

use wasm_bindgen::JsCast;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
    let view_clone: std::rc::Rc<std::cell::Cell<View>> = view.clone();

    log::debug!("begin");

    let f: std::rc::Rc<_> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g: std::rc::Rc<std::cell::RefCell<Option<_>>> = f.clone();

    *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {

        update(&webgpu_context, &mouse_event_js, &view_clone);
        rendering::webgpu::render(&webgpu_context);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    log::debug!("end");
}

fn request_animation_frame(f: &wasm_bindgen::closure::Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
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