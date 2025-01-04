#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
}

#[derive(Clone, Default)]
pub struct Mesh {
    pub _name: std::string::String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Option<u32>,
}
