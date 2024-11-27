mod engine;
mod frontend;
mod rendering;

use wasm_bindgen::JsCast;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen::prelude::wasm_bindgen(main)]
pub async fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    log::debug!("Main");

    // -------------------------------------------------------------------------

    let mouse_event_js: std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>> =
        std::rc::Rc::new(std::cell::Cell::new(
            frontend::controls::MouseEventResponseJs::default(),
        ));
    frontend::controls::add_event_listener_control(&mouse_event_js);

    // Model loading -----------------------------------------------------------
    // TODO: Multithread load, single is too slow
    
    let obj_loaded: (
        Vec<tobj::Model>,
        Result<Vec<tobj::Material>, tobj::LoadError>,
    ) = engine::load::load_mdl_async(engine::define::OBJ_BUNNY_PATH)
        .await
        .expect("Failed to load .mdl file");

    // -------------------------------------------------------------------------

    let update_context: std::rc::Rc<std::cell::Cell<engine::update::UpdateContext>> =
        std::rc::Rc::new(std::cell::Cell::new(engine::update::UpdateContext::initial()));
    let update_context_clone: std::rc::Rc<std::cell::Cell<engine::update::UpdateContext>> =
        update_context.clone();

    // Rendering  ---------------------------------------------------------------

    let mut webgpu_context: rendering::webgpu::WebGPUContext = rendering::webgpu::init().await;
    if !obj_loaded.0.is_empty() && !obj_loaded.0.first().unwrap().mesh.positions.is_empty() {
        rendering::webgpu::override_context(&mut webgpu_context, &obj_loaded.0.first().unwrap());
    }

    // Game loop ----------------------------------------------------------------

    let f: std::rc::Rc<_> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g: std::rc::Rc<std::cell::RefCell<Option<_>>> = f.clone();
    *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        engine::update::update(&webgpu_context, &mouse_event_js, &update_context_clone);
        engine::update::update_js_context(&mouse_event_js);

        rendering::webgpu::render(&webgpu_context);

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
