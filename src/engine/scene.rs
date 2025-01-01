use crate::{frontend, rendering};

use glam::Vec4Swizzles;

#[derive(Clone, Default)]
pub struct Scene {
    pub eye_location: glam::Vec3,
    pub eye_direction: glam::Vec3,
    pub directional_light_angle: [f32; 3],
    pub ambient_light_color: [f32; 4],
    pub background_color: [f32; 4],
    pub scene_shading_type: ShadingType,
    pub differed_debug_type: u8,
    pub objects: Vec<SceneObject>,
    pub instant_convert_y_to_z: bool,
    pub materials: Vec<SceneMaterial>,
}
impl Scene {
    pub fn init(&mut self) {
        self.eye_location = glam::Vec3 {
            x: 5.0,
            y: 0.0,
            z: 0.5,
        };
        self.eye_direction = -glam::Vec3::X;
        self.directional_light_angle = [0.0, 0.0, -1.0];
        self.ambient_light_color = [0.2, 0.2, 0.2, 1.0];
        self.background_color = [0.7, 0.7, 0.7, 1.0];
        self.scene_shading_type = ShadingType::Differed;
        self.differed_debug_type = 0;
        self.objects = Vec::new();
        self.instant_convert_y_to_z = true;
    }
}

#[derive(Clone, Default)]
pub struct SceneObject {
    pub _name: Option<std::string::String>,
    pub shading_type: u8,
    pub model_matrix: [[f32; 4]; 4],
    pub source_mesh: Option<std::rc::Rc<std::cell::RefCell<rendering::common::Mesh>>>,
    pub render_resource:
        Option<std::rc::Rc<std::cell::RefCell<rendering::webgpu::WebGPURenderResource>>>,
    pub index: u32,
    pub parent_index: Option<u32>,
    pub child_index: Vec<u32>,
    //pub material_index: Option<u32>,
}
#[derive(Clone, Default)]
pub struct SceneMaterial {
    pub _name: Option<std::string::String>,
    pub base_color_texture_dat: Vec<u8>,
    pub base_color_texture_size: [u32; 2],
}
#[derive(Clone, Copy, Default)]
pub enum ShadingType {
    #[default]
    None,
    Differed,
    Forward,
}

// ---------------------------------------------------------------------------------------

pub fn update_js(
    scene: &std::rc::Rc<std::cell::RefCell<Scene>>,
    in_control_event: &std::rc::Rc<std::cell::RefCell<frontend::eventlistener::ControlResponseJs>>,
) {
    let mut scene_value = scene.borrow_mut();
    let mut eye: glam::Vec3 = scene_value.eye_location;
    let mut direction: glam::Vec3 = scene_value.eye_direction;

    let mut control_event_js = in_control_event.borrow_mut();

    // Calculate eye direction (rotation)
    let on_click: bool = control_event_js.on_click;
    let on_shift: bool = control_event_js.on_shift;
    if on_click && !on_shift {
        let rotate_x_mat =
            glam::Mat3::from_rotation_z(-1.0 * control_event_js.movement_x as f32 * 0.005);
        direction = rotate_x_mat.mul_vec3(direction);

        let y_axis = glam::vec3(direction.x, direction.y, direction.z)
            .cross(glam::vec3(0.0, 0.0, 1.0))
            .normalize();
        let rotate_y_quat =
            glam::Quat::from_axis_angle(y_axis, -1.0 * control_event_js.movement_y as f32 * 0.005);
        direction = rotate_y_quat.mul_vec3(direction);
    } else if on_click && on_shift {
        let direction_mat: glam::Mat4 = glam::Mat4::from_translation(direction);
        let up_move_vec: glam::Vec4 = direction_mat.mul_vec4(glam::Vec4::Z).normalize();
        let right_move_vec: glam::Vec4 = direction_mat.mul_vec4(glam::Vec4::Y).normalize();
        eye += -1.0 * up_move_vec.xyz() * control_event_js.movement_y as f32 * 0.01;
        eye += 1.0 * right_move_vec.xyz() * control_event_js.movement_x as f32 * 0.01;
    }

    // Calculate eye location
    let on_wheel = control_event_js.on_wheel;
    if on_wheel {
        eye += -1.0 * direction.normalize() * control_event_js.wheel_delta_y as f32 * 0.005;
    }

    // Update
    scene_value.eye_location = eye;
    scene_value.eye_direction = direction;

    // Event context override
    control_event_js.on_click = false;
    control_event_js.on_wheel = false;
}
