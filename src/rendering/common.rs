pub const INITIAL_EYE_LOCATION: glam::Vec3 = glam::Vec3::new(1.5f32, -5.0, 3.0);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _color: [f32; 3],
    _uv: [f32; 2],
}

impl Vertex {
    pub fn new(pos: [f32; 3], color: [f32; 3], uv: [f32; 2]) -> Vertex {
        Vertex {
            _pos: [pos[0], pos[1], pos[2], 1.0],
            _color: color,
            _uv: uv,
        }
    }
}

pub fn create_cube() -> (Vec<Vertex>, Vec<u32>) {
    let vertex_data = [
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0, 0.0], [1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [1.0, 1.0, 0.0], [0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 1, 3, // top
        5, 4, 7, 7, 4, 6, // bottom
        1, 5, 3, 3, 5, 7, // front
        2, 6, 0, 0, 6, 4, // back
        3, 7, 2, 2, 7, 6, // right
        0, 4, 1, 1, 4, 5, // left
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

// -----------------------------------------------------------------------------

pub fn create_mvp(aspect_ratio: f32) -> glam::Mat4 {
    let projection =
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);

    let view = glam::Mat4::look_at_rh(INITIAL_EYE_LOCATION, glam::Vec3::ZERO, glam::Vec3::Z);

    projection * view
}

// Suports tobj format -> Vertex
pub fn create_vertices_from_obj(model: &tobj::Model, swap_yz: bool) -> Vec<Vertex> {
    let mut vertex_vec: Vec<Vertex> = Vec::new();

    for i in 0..(model.mesh.positions.len() / 3) {
        let mut pos: [f32; 3] = [
            model.mesh.positions[3 * i],
            model.mesh.positions[3 * i + 1],
            model.mesh.positions[3 * i + 2],
        ];
        let mut color = [0.0, 0.0, 0.0];
        if model.mesh.vertex_color.len() > i * 3 {
            color = [
                model.mesh.vertex_color[3 * i],
                model.mesh.vertex_color[3 * i + 1],
                model.mesh.vertex_color[3 * i + 2],
            ];
        }
        let uvs: [f32; 2] = [model.mesh.texcoords[i], model.mesh.texcoords[i + 1]];

        if swap_yz {
            pos = [pos[0], pos[2], pos[1]];
            color = [color[0], color[2], color[1]];
        }

        vertex_vec.push(Vertex {
            _pos: [pos[0], pos[1], pos[2], 1.0],
            _color: color,
            _uv: uvs,
        });
    }

    return vertex_vec;
}
