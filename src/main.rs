mod engine;
mod frontend;
mod rendering;

use engine::define::{OBJ_MORI_KNOB_PATH, OBJ_SPHERE_PATH};
use wasm_bindgen::JsCast;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen::prelude::wasm_bindgen(main)]
pub async fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::debug!("Main");

    // Js  ----------------------------------------------------------------------

    let mouse_event_js: std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>> =
        std::rc::Rc::new(std::cell::Cell::new(
            frontend::controls::MouseEventResponseJs::default(),
        ));
    frontend::controls::add_event_listener_control(&mouse_event_js);

    // Model loading  ----------------------------------------------------------

    // TODO: Multithread load
    let obj_meshes: Vec<rendering::common::Mesh> =
        engine::load::load_obj(engine::define::OBJ_MORI_KNOB_PATH).await;

    // Rendering  ---------------------------------------------------------------

    let webgpu_interface: rendering::webgpu::WebGPUInterface =
        rendering::webgpu::init_webgpu().await;

    let mut webgpu_resources: Vec<rendering::webgpu::WebGPURenderResource> = Vec::new();
    for obj_mesh in obj_meshes.iter() {
        let webgpu_resource =
            rendering::webgpu::init_webgpu_phong_shader(&webgpu_interface, &obj_mesh);
        webgpu_resources.push(webgpu_resource);
    }

    // Update variables  --------------------------------------------------------

    let scene: std::rc::Rc<std::cell::Cell<engine::update::Scene>> =
        std::rc::Rc::new(std::cell::Cell::new(engine::update::Scene::get_init()));
    let scene_clone: std::rc::Rc<std::cell::Cell<engine::update::Scene>> = scene.clone();

    // Game loop  ---------------------------------------------------------------

    let f: std::rc::Rc<_> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g: std::rc::Rc<std::cell::RefCell<Option<_>>> = f.clone();
    *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        engine::update::update_js(&scene_clone, &mouse_event_js);

        for webgpu_resource in webgpu_resources.iter() {
            rendering::webgpu::write_webgpu_phong_buffer(
                &scene_clone,
                &webgpu_interface,
                &webgpu_resource,
            );
        }
        rendering::webgpu::render_main(&webgpu_interface, &webgpu_resources);

        request_animation_frame(f.borrow().as_ref().unwrap());
    })
        as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());

    log::debug!("Main end");
}

fn request_animation_frame(f: &wasm_bindgen::closure::Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
