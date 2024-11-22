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

    log::debug!("begin");

    let webgpu_context: rendering::webgpu::WebGPUContext = rendering::webgpu::init().await;

    let mouse_event_js: std::rc::Rc<std::cell::Cell<frontend::controls::MouseEventResponseJs>> =
        std::rc::Rc::new(std::cell::Cell::new(
            frontend::controls::MouseEventResponseJs {
                movement_x: 0,
                movement_y: 0,
                on_click: false,
            },
        ));
    frontend::controls::add_event_listener_control(&mouse_event_js);

    let update_context: std::rc::Rc<std::cell::Cell<engine::define::UpdateContext>> =
        std::rc::Rc::new(std::cell::Cell::new(engine::define::UpdateContext {
            eye: glam::Vec3::new(1.5f32, -5.0, 3.0),
        }));
    let update_context_clone: std::rc::Rc<std::cell::Cell<engine::define::UpdateContext>> =
        update_context.clone();

    // ----------------------------------------------------------------

    let f: std::rc::Rc<_> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let g: std::rc::Rc<std::cell::RefCell<Option<_>>> = f.clone();

    *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        engine::update::update(&webgpu_context, &mouse_event_js, &update_context_clone);
        rendering::webgpu::render(&webgpu_context);

        request_animation_frame(f.borrow().as_ref().unwrap());
    })
        as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    log::debug!("end");
}

fn request_animation_frame(f: &wasm_bindgen::closure::Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
