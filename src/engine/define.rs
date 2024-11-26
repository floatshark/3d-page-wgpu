// frontend
pub const CANVAS_ELEMENT_ID: &str = "canvas";
// rendering
pub const VS_ENTRY_POINT: &str = "vs_main";
pub const FS_ENTRY_POINT: &str = "fs_main";
// load
pub const OBJ_BUNNY_PATH: &str = "resource/bunny/bunny.obj";

#[derive(Clone, Copy)]
pub struct UpdateContext {
    pub eye: glam::Vec3,
}
