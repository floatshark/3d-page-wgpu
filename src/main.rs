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

    /*let obj_loaded: (
        Vec<tobj::Model>,
        Result<Vec<tobj::Material>, tobj::LoadError>,
    ) = engine::load::load_mdl_async(engine::define::OBJ_BUNNY_PATH)
        .await
        .expect("Failed to load .mdl file");
        */
    // -------------------------------------------------------------------------

    let update_context: std::rc::Rc<std::cell::Cell<engine::update::UpdateContext>> =
        std::rc::Rc::new(std::cell::Cell::new(
            engine::update::UpdateContext::get_init(),
        ));
    let update_context_clone: std::rc::Rc<std::cell::Cell<engine::update::UpdateContext>> =
        update_context.clone();

    // Rendering  ---------------------------------------------------------------

    let webgpu_interface: rendering::webgpu::WebGPUInterface =
        rendering::webgpu::init_webgpu().await;
    let webgpu_resource = rendering::webgpu::init_webgpu_color_shader(&webgpu_interface);
    
    // Game loop ----------------------------------------------------------------

    let f: std::rc::Rc<_> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g: std::rc::Rc<std::cell::RefCell<Option<_>>> = f.clone();
    *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        engine::update::update_js(&mouse_event_js, &update_context_clone);
        engine::update::update_render_resource(
            &update_context_clone,
            &webgpu_interface,
            &webgpu_resource,
        );

        rendering::webgpu::render_main(&webgpu_interface, &webgpu_resource);

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
