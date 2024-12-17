use crate::frontend;

use glam::Vec4Swizzles;

#[derive(Clone, Copy, Default)]
pub struct Scene {
    pub eye_location: glam::Vec3,
    pub eye_direction: glam::Vec3,
    pub directional_light_angle: [f32; 3],
    pub ambient_light_color: [f32; 4],
    pub background_color: [f32; 4],
    pub render_type: u8,
    pub differed_debug_type: u8,
}
impl Scene {
    pub fn get_init() -> Scene {
        Scene {
            eye_location: glam::Vec3 {
                x: 5.0,
                y: 0.0,
                z: 0.5,
            },
            eye_direction: -glam::Vec3::X,
            directional_light_angle: [0.0, 0.0, -1.0],
            ambient_light_color: [0.2, 0.2, 0.2, 1.0],
            background_color: [0.7, 0.7, 0.7, 1.0],
            render_type: 0,
            differed_debug_type: 0,
        }
    }
}

// ---------------------------------------------------------------------------------------

pub fn update_js(
    scene: &std::rc::Rc<std::cell::Cell<Scene>>,
    mouse_event_js: &std::rc::Rc<std::cell::Cell<frontend::eventlistener::MouseEventResponseJs>>,
) {
    let mut eye: glam::Vec3 = scene.get().eye_location;
    let mut direction: glam::Vec3 = scene.get().eye_direction;

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

    // Update
    let mut scene_updated: Scene = scene.get();
    scene_updated.eye_location = eye;
    scene_updated.eye_direction = direction;
    scene.set(scene_updated);

    // Event context override
    let mut override_event: frontend::eventlistener::MouseEventResponseJs = mouse_event_js.get();
    override_event.on_click = false;
    override_event.on_wheel = false;
    mouse_event_js.set(override_event);
}
