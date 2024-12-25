#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],
    _color: [f32; 3],
    _uv: [f32; 2],
    _normal: [f32; 3],
}

impl Vertex {
    pub fn new(pos: [f32; 3], color: [f32; 3], uv: [f32; 2], normal: [f32; 3]) -> Vertex {
        Vertex {
            _pos: [pos[0], pos[1], pos[2], 1.0],
            _color: color,
            _uv: uv,
            _normal: normal,
        }
    }
}

#[derive(Clone, Default)]
pub struct Mesh {
    pub _name: std::string::String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Option<u32>,
}

// -----------------------------------------------------------------------------

#[allow(dead_code)]
pub fn create_cube() -> Mesh {
    let vertex_data = [
        // top (0, 0, 1)
        Vertex::new(
            [-1.0, -1.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0, 1.0],
        ),
        Vertex::new(
            [1.0, -1.0, 1.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0],
            [0.0, 0.0, 1.0],
        ),
        Vertex::new(
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0],
            [0.0, 0.0, 1.0],
        ),
        Vertex::new(
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0],
            [0.0, 0.0, 1.0],
        ),
        // bottom (0, 0, -1.0)
        Vertex::new(
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0],
            [0.0, 0.0, -1.0],
        ),
        Vertex::new(
            [1.0, 1.0, -1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0],
            [0.0, 0.0, -1.0],
        ),
        Vertex::new(
            [1.0, -1.0, -1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0],
            [0.0, 0.0, -1.0],
        ),
        Vertex::new(
            [-1.0, -1.0, -1.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0],
            [0.0, 0.0, -1.0],
        ),
        // forward (1.0, 0, 0)
        Vertex::new(
            [1.0, -1.0, -1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0],
            [1.0, 0.0, 0.0],
        ),
        Vertex::new(
            [1.0, 1.0, -1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0],
            [1.0, 0.0, 0.0],
        ),
        Vertex::new(
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0, 0.0],
        ),
        Vertex::new(
            [1.0, -1.0, 1.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.0, 0.0],
        ),
        // back (-1.0, 0, 0)
        Vertex::new(
            [-1.0, -1.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ),
        Vertex::new(
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0],
            [-1.0, 0.0, 0.0],
        ),
        Vertex::new(
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0],
            [-1.0, 0.0, 0.0],
        ),
        Vertex::new(
            [-1.0, -1.0, -1.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0],
            [-1.0, 0.0, 0.0],
        ),
        // right (0, 1.0, 0)
        Vertex::new(
            [1.0, 1.0, -1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0],
            [0.0, 1.0, 0.0],
        ),
        Vertex::new(
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0, 0.0],
        ),
        Vertex::new(
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0],
            [0.0, 1.0, 0.0],
        ),
        Vertex::new(
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0],
            [0.0, 1.0, 0.0],
        ),
        // left (0, -1.0, 0)
        Vertex::new(
            [1.0, -1.0, 1.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0, 0.0],
        ),
        Vertex::new(
            [-1.0, -1.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0],
            [0.0, 1.0, 0.0],
        ),
        Vertex::new(
            [-1.0, -1.0, -1.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0, 0.0],
        ),
        Vertex::new(
            [1.0, -1.0, -1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0],
            [0.0, 1.0, 0.0],
        ),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    Mesh {
        _name: "cube".to_string(),
        vertices: vertex_data.to_vec(),
        indices: index_data.to_vec(),
        material: None,
    }
}

// -----------------------------------------------------------------------------

// Supports tobj format -> Vertex Vec
pub fn create_vertices_from_obj(model: &tobj::Model, swap_yz: bool) -> Vec<Vertex> {
    let mut vertex_vec: Vec<Vertex> = Vec::new();

    for i in 0..(model.mesh.positions.len() / 3) {
        let mut pos: [f32; 3] = [
            model.mesh.positions[3 * i],
            model.mesh.positions[3 * i + 1],
            model.mesh.positions[3 * i + 2],
        ];
        let mut color: [f32; 3] = [0.0, 0.0, 0.0];
        if model.mesh.vertex_color.len() > i * 3 {
            color = [
                model.mesh.vertex_color[3 * i],
                model.mesh.vertex_color[3 * i + 1],
                model.mesh.vertex_color[3 * i + 2],
            ];
        }
        let mut uvs: [f32; 2] = [0.0, 0.0];
        if model.mesh.texcoords.len() > i * 2 {
            uvs = [model.mesh.texcoords[2 * i], model.mesh.texcoords[2 * i + 1]];
        }
        let mut normal: [f32; 3] = [0.0, 0.0, 0.0];
        if model.mesh.normals.len() > i * 3 {
            normal = [
                model.mesh.normals[3 * i],
                model.mesh.normals[3 * i + 1],
                model.mesh.normals[3 * i + 2],
            ];
        }

        if swap_yz {
            pos = [pos[0], pos[2], pos[1]];
            color = [color[0], color[2], color[1]];
            normal = [normal[0], normal[2], normal[1]];
        }

        vertex_vec.push(Vertex {
            _pos: [pos[0], pos[1], pos[2], 1.0],
            _color: color,
            _uv: uvs,
            _normal: normal,
        });
    }

    return vertex_vec;
}

// Supports tobj format -> indices Vec
pub fn create_indices_from_obj(model: &tobj::Model, swap_yz: bool) -> Vec<u32> {
    let mut indices_vec: Vec<u32> = Vec::new();

    for i in 0..(model.mesh.indices.len() / 3) {
        let mut index: [u32; 3] = [
            model.mesh.indices[3 * i],
            model.mesh.indices[3 * i + 1],
            model.mesh.indices[3 * i + 2],
        ];

        if swap_yz {
            index = [index[0], index[2], index[1]];
        }

        indices_vec.push(index[0]);
        indices_vec.push(index[1]);
        indices_vec.push(index[2]);
    }
    return indices_vec;
}
