use crate::engine::define;
use crate::frontend;
use crate::rendering;

use glam::Vec4Swizzles;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Default)]
pub struct UpdateContext {
    pub eye_location: glam::Vec3,
    pub eye_direction: glam::Vec3,
}

impl UpdateContext {
    pub fn get_init() -> UpdateContext {
        UpdateContext {
            eye_location: glam::Vec3 {
                x: 5.0,
                y: 0.0,
                z: 0.5,
            },
            eye_direction: -glam::Vec3::X,
        }
    }
}

pub fn update(
    render_context: &rendering::webgpu::WebGPUContext,
    mouse_event_js: &std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>>,
    context: &std::rc::Rc<std::cell::Cell<UpdateContext>>,
) {
    let mut eye: glam::Vec3 = context.get().eye_location;
    let mut direction: glam::Vec3 = context.get().eye_direction;

    let canvas: web_sys::Element = gloo::utils::document()
        .get_element_by_id(define::CANVAS_ELEMENT_ID)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let width: u32 = canvas.client_width() as u32;
    let height: u32 = canvas.client_height() as u32;
    let aspect_ratio: f32 = width as f32 / height as f32;

    // Calculate eye direction (rotation)
    let on_click: bool = mouse_event_js.get().on_click;
    let on_shift: bool = mouse_event_js.get().on_shift;
    if on_click && !on_shift {
        let rotate_x_mat =
            glam::Mat3::from_rotation_z(-1.0 * mouse_event_js.get().movement_x as f32 * 0.005);
        direction = rotate_x_mat.mul_vec3(direction);

        let y_axis = glam::vec3(direction.x, direction.y, direction.z)
            .cross(glam::vec3(0.0, 0.0, 1.0))
            .normalize();
        let rotate_y_quat = glam::Quat::from_axis_angle(
            y_axis,
            -1.0 * mouse_event_js.get().movement_y as f32 * 0.005,
        );
        direction = rotate_y_quat.mul_vec3(direction);
    } else if on_click && on_shift {
        let direction_mat: glam::Mat4 = glam::Mat4::from_translation(direction);
        let up_move_vec: glam::Vec4 = direction_mat.mul_vec4(glam::Vec4::Z).normalize();
        let right_move_vec: glam::Vec4 = direction_mat.mul_vec4(glam::Vec4::Y).normalize();
        eye += -1.0 * up_move_vec.xyz() * mouse_event_js.get().movement_y as f32 * 0.01;
        eye += 1.0 * right_move_vec.xyz() * mouse_event_js.get().movement_x as f32 * 0.01;
    }

    // Calculate eye location
    let on_wheel = mouse_event_js.get().on_wheel;
    if on_wheel {
        eye += -1.0 * direction.normalize() * mouse_event_js.get().wheel_delta_y as f32 * 0.005;
    }

    // Create matrices and write buffer
    let view_matrix = glam::Mat4::look_to_rh(eye, direction, glam::Vec3::Z);
    let projection_matrix: glam::Mat4 =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.0, 10.0);
    let mx_total: glam::Mat4 = projection_matrix * view_matrix;
    let mx_ref: &[f32; 16] = mx_total.as_ref();
    render_context
        .queue
        .write_buffer(&render_context.uniform_buf, 0, bytemuck::cast_slice(mx_ref));

    // Update
    let view_temp: UpdateContext = UpdateContext {
        eye_location: eye,
        eye_direction: direction,
    };
    context.set(view_temp);
}

pub fn update_js_context(
    mouse_event_js: &std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>>,
) {
    let mut override_event: frontend::controls::MouseEventResponseJs = mouse_event_js.get();
    override_event.on_click = false;
    override_event.on_wheel = false;
    mouse_event_js.set(override_event);
}
