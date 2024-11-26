use crate::engine::define;
use crate::frontend;
use crate::rendering;

use wasm_bindgen::JsCast;

pub fn update(
    render_context: &rendering::webgpu::WebGPUContext,
    mouse_event_js: &std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>>,
    view: &std::rc::Rc<std::cell::Cell<define::UpdateContext>>,
) {
    let mut eye: glam::Vec3 = view.get().eye;

    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
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

    let view_temp: define::UpdateContext = define::UpdateContext { eye: eye };
    view.set(view_temp);
}

pub fn update_mdl_load(
    mdl_reveiver: &std::sync::mpsc::Receiver<(
        Vec<tobj::Model>,
        Result<Vec<tobj::Material>, tobj::LoadError>,
    )>,
) {
    if mdl_reveiver.try_recv().is_ok(){
        log::debug!("model loaded");
    }
}
