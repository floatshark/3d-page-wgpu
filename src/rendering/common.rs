pub const INITIAL_EYE_LOCATION: glam::Vec3 = glam::Vec3::new(1.5f32, -5.0, 3.0);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _uv: [f32; 2],
}

impl Vertex {
    pub fn new(pos: [f32; 3], uv: [f32; 2]) -> Vertex {
        Vertex {
            _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
            _uv: [uv[0] as f32, uv[1] as f32],
        }
    }
}

pub fn create_cube() -> (Vec<Vertex>, Vec<u32>) {
    let vertex_data = [
        // top (0, 0, 1)
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 1.0]),
        // bottom (0, 0, -1)
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // front (1, 0, 0)
        Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0]),
        // back (-1, 0, 0)
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 1.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 0.0]),
        // right (0, 1, 0)
        Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 1.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0]),
        // left (0, -1, 0)
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 1.0]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
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
        if !swap_yz {
            vertex_vec.push(Vertex {
                _pos: [
                    model.mesh.positions[3 * i],
                    model.mesh.positions[3 * i + 1],
                    model.mesh.positions[3 * i + 2],
                    1.0,
                ],
                _uv: [model.mesh.texcoords[i], model.mesh.texcoords[i + 1]],
            });
        } else {
            vertex_vec.push(Vertex {
                _pos: [
                    model.mesh.positions[3 * i],
                    model.mesh.positions[3 * i + 2],
                    model.mesh.positions[3 * i + 1],
                    1.0,
                ],
                _uv: [model.mesh.texcoords[i], model.mesh.texcoords[i + 1]],
            });
        }
    }

    return vertex_vec;
}
