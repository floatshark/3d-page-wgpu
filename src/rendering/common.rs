pub const INITIAL_EYE_LOCATION: glam::Vec3 = glam::Vec3::new(1.5f32, -5.0, 3.0);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    _pos: [f32; 4],    
    _uv : [f32; 2],
}

impl Vertex
{
    pub fn vertex(pos: [i8; 3], uv: [f32; 2]) -> Vertex {
        Vertex {
            _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
            _uv : [uv[0]as f32, uv[1] as f32]
        }
    }
}

pub fn create_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        Vertex::vertex([-1, -1, 1], [0.0, 0.0]),
        Vertex::vertex([1, -1, 1], [1.0, 0.0]),
        Vertex::vertex([1, 1, 1], [0.0, 1.0]),
        Vertex::vertex([-1, 1, 1], [1.0, 1.0]),
        // bottom (0, 0, -1)
        Vertex::vertex([-1, 1, -1], [0.0, 0.0]),
        Vertex::vertex([1, 1, -1], [1.0, 0.0]),
        Vertex::vertex([1, -1, -1], [0.0, 1.0]),
        Vertex::vertex([-1, -1, -1], [1.0, 1.0]),
        // right (1, 0, 0)
        Vertex::vertex([1, -1, -1], [0.0, 0.0]),
        Vertex::vertex([1, 1, -1], [0.0, 1.0]),
        Vertex::vertex([1, 1, 1], [1.0, 0.0]),
        Vertex::vertex([1, -1, 1], [1.0, 1.0]),
        // left (-1, 0, 0)
        Vertex::vertex([-1, -1, 1], [0.0, 0.0]),
        Vertex::vertex([-1, 1, 1], [1.0, 0.0]),
        Vertex::vertex([-1, 1, -1], [0.0, 1.0]),
        Vertex::vertex([-1, -1, -1], [1.0, 1.0]),
        // front (0, 1, 0)
        Vertex::vertex([1, 1, -1], [0.0, 0.0]),
        Vertex::vertex([-1, 1, -1], [1.0, 0.0]),
        Vertex::vertex([-1, 1, 1], [0.0, 1.0]),
        Vertex::vertex([1, 1, 1], [1.0, 0.0]),
        // back (0, -1, 0)
        Vertex::vertex([1, -1, 1], [0.0, 0.0]),
        Vertex::vertex([-1, -1, 1], [1.0, 0.0]),
        Vertex::vertex([-1, -1, -1], [0.0, 1.0]),
        Vertex::vertex([1, -1, -1], [1.0, 1.0]),
    ];

    let index_data: &[u16] = &[
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
    
    let view = glam::Mat4::look_at_rh(
        INITIAL_EYE_LOCATION,
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    
    projection * view
}
