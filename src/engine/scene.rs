use crate::{frontend, rendering};

use glam::Vec4Swizzles;

#[derive(Clone, Default)]
pub struct Scene {
    // own
    pub objects: Vec<SceneObject>,
    pub batched_objects: Vec<SceneObject>,
    pub materials: Vec<SceneMaterial>,
    // world variables
    pub eye_location: glam::Vec3,
    pub eye_direction: glam::Vec3,
    pub directional_light_angle: [f32; 3],
    pub ambient_light_color: [f32; 4],
    pub background_color: [f32; 4],
    // configs
    pub is_first_update: bool,
    pub convert_y_to_z: bool,
    pub scene_shading_type: ShadingType,
    pub differed_debug_type: u8,
    pub use_batched: bool,
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
        self.ambient_light_color = [0.0, 0.0, 0.0, 1.0];
        self.background_color = [0.7, 0.7, 0.7, 1.0];
        self.scene_shading_type = ShadingType::Differed;
        self.differed_debug_type = 0;
        self.objects = Vec::new();
        self.convert_y_to_z = true;
        self.is_first_update = true;
        self.use_batched = true;
    }
}

#[derive(Clone, Default)]
pub struct SceneObject {
    pub _name: Option<std::string::String>,
    pub index: u32,
    pub parent_index: Option<u32>,
    pub child_index: Vec<u32>,
    pub world_transform: [[f32; 4]; 4],
    pub source_mesh: Option<std::rc::Rc<std::cell::RefCell<rendering::common::Mesh>>>,
    pub shading_type: u8,
    pub render_resource:
        Option<std::rc::Rc<std::cell::RefCell<rendering::webgpu::WebGPURenderResource>>>,
}

#[derive(Clone, Default)]
pub struct SceneMaterial {
    pub _name: Option<std::string::String>,
    pub base_color_texture: Vec<u8>,
    pub base_color_texture_size: [u32; 2],
    pub normal_texture: Vec<u8>,
    pub normal_texture_size: [u32; 2],
    pub metallic_roughness_texture: Vec<u8>,
    pub metallic_roughness_texture_size: [u32; 2],
}

#[derive(Clone, Copy, Default)]
pub enum ShadingType {
    #[default]
    None,
    Differed,
    Forward,
}

// Util

pub fn batch_objects(scene: &std::rc::Rc<std::cell::RefCell<Scene>>) {
    let mut batch_map: std::collections::HashMap<u32, rendering::common::Mesh> =
        std::collections::HashMap::with_capacity(scene.borrow().objects.len());
    for object in scene.borrow().objects.iter() {
        if object.source_mesh.is_some() {
            let source_mesh = object.source_mesh.as_ref().unwrap();
            let material_option = source_mesh.borrow().material;
            if material_option.is_some() {
                let material = material_option.as_ref().unwrap();
                // init
                if batch_map.get(material).is_none() {
                    batch_map.insert(*material, rendering::common::Mesh::default());
                }
                // batch
                let batched_mesh = batch_map.get_mut(material).unwrap();
                let mut source_vertices = source_mesh.borrow().vertices.clone();
                let indices_offset = batched_mesh.vertices.len() as u32;
                let trans_matrix = glam::Mat4::from_cols_array_2d(&object.world_transform);
                let rotation_matrix =
                    glam::Mat4::from_quat(trans_matrix.to_scale_rotation_translation().1);
                for i in 0..source_vertices.len() {
                    let vert = glam::Vec4::from_array(source_vertices[i].pos);
                    let transed_vert = trans_matrix.mul_vec4(vert);
                    source_vertices[i].pos = transed_vert.to_array();
                    let norm = glam::Vec4::new(
                        source_vertices[i].normal[0],
                        source_vertices[i].normal[1],
                        source_vertices[i].normal[2],
                        1.0,
                    );
                    let transed_norm = rotation_matrix.mul_vec4(norm);
                    source_vertices[i].normal = [transed_norm.x, transed_norm.y, transed_norm.z];
                    let tangent = glam::Vec4::new(
                        source_vertices[i].tangent[0],
                        source_vertices[i].tangent[1],
                        source_vertices[i].tangent[2],
                        1.0,
                    );
                    let transed_tangent = rotation_matrix.mul_vec4(tangent);
                    source_vertices[i].tangent =
                        [transed_tangent.x, transed_tangent.y, transed_tangent.z];
                }
                batched_mesh.vertices.append(&mut source_vertices);

                let mut source_indices = source_mesh.borrow().indices.clone();
                for index in source_indices.iter_mut() {
                    *index += indices_offset;
                }
                batched_mesh.indices.append(&mut source_indices);
                batched_mesh.material = Some(*material);
            } else {
                // no material not create batched mesh
            }
        }
    }

    for batch_pair in batch_map {
        let batched_object = SceneObject {
            _name: Some("batched".to_string()),
            shading_type: 44,
            world_transform: glam::Mat4::IDENTITY.to_cols_array_2d(),
            source_mesh: Some(std::rc::Rc::new(std::cell::RefCell::new(batch_pair.1))),
            render_resource: None,
            index: batch_pair.0,
            ..Default::default()
        };
        scene.borrow_mut().batched_objects.push(batched_object);
    }
}

pub fn update_control(
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
