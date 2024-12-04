use crate::rendering;

pub fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();
    /*if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }*/
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub async fn _load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

pub async fn load_obj_async(file_name: &str) -> rendering::common::Mesh {
    let obj_text = load_string(file_name)
        .await
        .expect("Failed to parse object name string");
    let obj_cursor: std::io::Cursor<String> = std::io::Cursor::new(obj_text);
    let mut obj_reader: std::io::BufReader<_> = std::io::BufReader::new(obj_cursor);

    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let loaded_obj = tobj::load_obj_buf_async(&mut obj_reader, &load_options, |p| async move {
        let mat_text = load_string(&p).await.unwrap();
        tobj::load_mtl_buf(&mut std::io::BufReader::new(std::io::Cursor::new(mat_text)))
    })
    .await
    .expect("Failed to load obj");

    let model: &tobj::Model = loaded_obj.0.first().expect("Failed to get first model");

    log::debug!(
        "loaded {} : vertices {}, indices {}",
        file_name,
        model.mesh.positions.len() / 3,
        model.mesh.indices.len()
    );

    rendering::common::Mesh {
        _name: model.name.clone(),
        vertices: rendering::common::create_vertices_from_obj(&model, true),
        indices: rendering::common::create_indices_from_obj(&model, true),
    }
}
