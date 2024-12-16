use crate::engine;
use wasm_bindgen::JsCast;

pub fn start_gui(scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>) {
    create_graphics_panel(scene);
}

fn create_graphics_panel(scene: &std::rc::Rc<std::cell::Cell<engine::update::Scene>>) {
    let body: web_sys::HtmlElement = gloo::utils::body();

    let graphics_element: web_sys::Element = gloo::utils::document().create_element("div").unwrap();
    graphics_element.set_id("left-panel");

    let accordion_input_element = gloo::utils::document().create_element("input").unwrap();
    let accordion_input_element: web_sys::HtmlInputElement =
        accordion_input_element.dyn_into().unwrap();
    accordion_input_element
        .set_attribute("type", "checkbox")
        .unwrap();
    accordion_input_element.set_class_name("accordion-input");
    accordion_input_element.set_id("accordion-graphics");
    accordion_input_element.set_checked(true);

    let accordion_label_element = gloo::utils::document().create_element("label").unwrap();
    accordion_label_element.set_class_name("accordion-label");
    accordion_label_element.set_text_content(Some("Graphics"));
    accordion_label_element
        .set_attribute("for", "accordion-graphics")
        .unwrap();

    let accordion_content_element = gloo::utils::document().create_element("div").unwrap();
    accordion_content_element.set_class_name("accordion-content");

    // render type
    {
        let render_type_element: web_sys::Element =
            gloo::utils::document().create_element("div").unwrap();
        render_type_element.set_class_name("widget-row");

        let render_type_label_element: web_sys::Element =
            gloo::utils::document().create_element("div").unwrap();
        render_type_label_element.set_class_name("widget-label");
        render_type_label_element.set_text_content(Some("Render type"));

        let render_type_select_element = gloo::utils::document().create_element("select").unwrap();
        render_type_select_element.set_class_name("widget-value select-element");
        render_type_select_element.set_id("render-type-select");

        let render_type_option_differed = gloo::utils::document().create_element("option").unwrap();
        render_type_option_differed.set_node_value(Some("differed"));
        render_type_option_differed.set_text_content(Some("differed"));
        let render_type_option_forward = gloo::utils::document().create_element("option").unwrap();
        render_type_option_forward.set_node_value(Some("forward"));
        render_type_option_forward.set_text_content(Some("forward"));

        {
            let scene_clone: std::rc::Rc<std::cell::Cell<engine::update::Scene>> = scene.clone();
            let render_type_closure: wasm_bindgen::prelude::Closure<dyn FnMut(_)> =
                wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::InputEvent| {
                    let render_type_element: web_sys::Element = gloo::utils::document()
                        .get_element_by_id("render-type-select")
                        .unwrap();
                    let render_type_element: web_sys::HtmlSelectElement =
                        render_type_element.dyn_into().unwrap();
                    let value: String = render_type_element.value();

                    let mut scene_value: engine::update::Scene = scene_clone.get();
                    match value.as_str() {
                        "differed" => scene_value.render_type = 0,
                        "forward" => scene_value.render_type = 1,
                        _ => scene_value.render_type = 0,
                    }
                    scene_clone.set(scene_value);
                }) as Box<dyn FnMut(_)>);

            render_type_select_element
                .add_event_listener_with_callback(
                    "change",
                    render_type_closure.as_ref().unchecked_ref(),
                )
                .unwrap();
            render_type_closure.forget();
        }

        render_type_select_element
            .append_child(&render_type_option_differed)
            .unwrap();
        render_type_select_element
            .append_child(&render_type_option_forward)
            .unwrap();

        render_type_element
            .append_child(&render_type_label_element)
            .unwrap();
        render_type_element
            .append_child(&render_type_select_element)
            .unwrap();

        accordion_content_element.append_child(&render_type_element).unwrap();
    }

	// clear color
	{
		let clearcolor_element: web_sys::Element =
			gloo::utils::document().create_element("div").unwrap();
		clearcolor_element.set_class_name("widget-row");

		let clearcolor_label_element: web_sys::Element =
			gloo::utils::document().create_element("div").unwrap();
		clearcolor_label_element.set_class_name("widget-label");
		clearcolor_label_element.set_text_content(Some("Clear color"));

		let clearcolor_picker_element: web_sys::Element =
			gloo::utils::document().create_element("input").unwrap();
		clearcolor_picker_element.set_class_name("widget-value color-picker-element");
		clearcolor_picker_element.set_id("background-color-picker");
		clearcolor_picker_element
			.set_attribute("type", "color")
			.unwrap();
		{
			let bg_color: [f32; 4] = scene.get().background_color;
			let r_uint: u32 = (bg_color[0] * 255.0) as u32;
			let r_hex: String = format!("{r_uint:X}");
			let g_uint: u32 = (bg_color[1] * 255.0) as u32;
			let g_hex: String = format!("{g_uint:X}");
			let b_uint: u32 = (bg_color[2] * 255.0) as u32;
			let b_hex: String = format!("{b_uint:X}");

			let hex_string: String = "#".to_string() + &r_hex + &g_hex + &b_hex;
			clearcolor_picker_element
				.set_attribute("value", &hex_string)
				.unwrap();
		}

		{
			let scene_clone: std::rc::Rc<std::cell::Cell<engine::update::Scene>> = scene.clone();
			let bgcolor_picker_closure: wasm_bindgen::prelude::Closure<dyn FnMut(_)> =
				wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::InputEvent| {
					let picker_element: web_sys::Element = gloo::utils::document()
						.get_element_by_id("background-color-picker")
						.unwrap();
					let picker_element: web_sys::HtmlInputElement =
						picker_element.dyn_into().unwrap();
					let value: String = picker_element.value();

					let color_hex = value.trim_start_matches("#");
					let color_u8: [u8; 4] =
						u32::from_str_radix(&color_hex, 16).unwrap().to_be_bytes();

					let mut scene_value: engine::update::Scene = scene_clone.get();
					scene_value.background_color = [
						color_u8[1] as f32 / 256 as f32,
						color_u8[2] as f32 / 256 as f32,
						color_u8[3] as f32 / 256 as f32,
						1.0,
					];
					scene_clone.set(scene_value);
				}) as Box<dyn FnMut(_)>);

			clearcolor_picker_element
				.add_event_listener_with_callback(
					"input",
					bgcolor_picker_closure.as_ref().unchecked_ref(),
				)
				.unwrap();
			bgcolor_picker_closure.forget();
		}

		clearcolor_element
			.append_child(&clearcolor_label_element)
			.unwrap();
		clearcolor_element
			.append_child(&clearcolor_picker_element)
			.unwrap();

		accordion_content_element.append_child(&clearcolor_element).unwrap();
	}

    graphics_element.append_child(&accordion_input_element).unwrap();
    graphics_element.append_child(&accordion_label_element).unwrap();
    graphics_element
        .append_child(&accordion_content_element)
        .unwrap();

    body.append_child(&graphics_element).unwrap();
}
