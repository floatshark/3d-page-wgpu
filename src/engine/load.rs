use crate::engine;
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

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
            log::debug!("Load {} byte from {}", data.len(), file_name);
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

// .gltf

#[allow(dead_code)]
pub async fn load_gltf_scene(file_name: &str) -> Vec<engine::scene::SceneObject> {
    let gltf_text = load_string(file_name)
        .await
        .expect("Failed to parse .gltf file path string");
    let gltf_cursor: std::io::Cursor<String> = std::io::Cursor::new(gltf_text);
    let gltf_reader: std::io::BufReader<_> = std::io::BufReader::new(gltf_cursor);

    let gltf: gltf::Gltf = gltf::Gltf::from_reader(gltf_reader).expect("Failed to read .gltf file");
    let mut buffer_data: Vec<Vec<u8>> = Vec::new();

    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                //if let Some(blob) = gltf.blob.as_deref() {
                //    buffer_data.push(blob.into());
                //    log::debug!("Found a bin, saving");
                //};
            }
            gltf::buffer::Source::Uri(uri) => {
                let slash_num: usize = file_name.rfind("/").unwrap() + 1;
                let folder_path = file_name.split_at(slash_num).0;
                let binary_path = folder_path.to_string() + uri;
                let bin = load_binary(&binary_path.as_str())
                    .await
                    .expect("Failed to load binary");
                buffer_data.push(bin);
            }
        }
    }

    let mut out: Vec<engine::scene::SceneObject> = Vec::new();
    let mut num_node: u32 = 0;
    let mut num_verts: u32 = 0;
    let mut num_indices: u32 = 0;

    for _scene in gltf.scenes() {
        for node in gltf.nodes() {
            //log::debug!("Node : {}", node.name().unwrap());

            let mut mesh: Option<rendering::common::Mesh> = None;
            if node.mesh().is_some() {
                mesh = Some(get_gltf_mesh_from_node(&node, &buffer_data));
            }

            // for log
            num_node += 1;
            if mesh.is_some() {
                num_verts += mesh.as_ref().unwrap().vertices.len() as u32;
                num_indices += mesh.as_ref().unwrap().indices.len() as u32;
            }

            let mut scene_object = engine::scene::SceneObject {
                _name: Some(node.name().unwrap().to_string()),
                shading_type: 44,
                model_matrix: node.transform().matrix(),
                source_mesh: if mesh.is_some() {
                    Some(std::rc::Rc::new(std::cell::RefCell::new(mesh.unwrap())))
                } else {
                    None
                },
                render_resource: None,
                index: node.index() as u32,
                ..Default::default()
            };

            for child in node.children().into_iter() {
                scene_object.child_index.push(child.index() as u32);
            }

            out.push(scene_object);
        }
    }

    // Build parent relation, parent is unique
    let mut parent_vec: Vec<Option<u32>> = vec![None; out.len()];
    for object in out.iter() {
        for child in object.child_index.iter() {
            let inner = parent_vec.get_mut(*child as usize).unwrap();
            inner.replace(object.index);
        }
    }
    for i in 0..parent_vec.len() {
        let parent = parent_vec.get(i).unwrap();
        if parent.is_some() {
            let object = out.get_mut(i).unwrap();
            object.parent_index = Some(parent.unwrap());
        }
    }

    log::debug!(
        "nodes : {}, verts : {}, tris : {}",
        num_node,
        num_verts,
        num_indices / 3
    );

    return out;
}

#[allow(dead_code)]
pub async fn load_gltf_meshes(file_name: &str) -> Vec<rendering::common::Mesh> {
    let gltf_text = load_string(file_name)
        .await
        .expect("Failed to parse .gltf file path string");
    let gltf_cursor: std::io::Cursor<String> = std::io::Cursor::new(gltf_text);
    let gltf_reader: std::io::BufReader<_> = std::io::BufReader::new(gltf_cursor);

    let gltf: gltf::Gltf = gltf::Gltf::from_reader(gltf_reader).expect("Failed to read .gltf file");
    let mut buffer_data: Vec<Vec<u8>> = Vec::new();

    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                //if let Some(blob) = gltf.blob.as_deref() {
                //    buffer_data.push(blob.into());
                //    log::debug!("Found a bin, saving");
                //};
            }
            gltf::buffer::Source::Uri(uri) => {
                let slash_num: usize = file_name.rfind("/").unwrap() + 1;
                let folder_path = file_name.split_at(slash_num).0;
                let binary_path = folder_path.to_string() + uri;
                let bin = load_binary(&binary_path.as_str())
                    .await
                    .expect("Failed to load binary");
                buffer_data.push(bin);
            }
        }
    }

    let mut out: Vec<rendering::common::Mesh> = Vec::new();

    for _scene in gltf.scenes() {
        for node in gltf.nodes() {
            log::debug!("Node {} : {}", node.index(), node.name().unwrap());

            if node.mesh().is_none() {
                continue;
            }
            out.push(get_gltf_mesh_from_node(&node, &buffer_data));
        }
    }

    return out;
}

fn get_gltf_mesh_from_node(
    node: &gltf::Node<'_>,
    buffer_data: &Vec<Vec<u8>>,
) -> rendering::common::Mesh {
    let mesh = node.mesh().expect("Got mesh");
    let mut mesh_vertices: Vec<rendering::common::Vertex> = Vec::new();
    let mut mesh_indices: Vec<u32> = Vec::new();

    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

        if reader.read_positions().is_none() {
            continue;
        }

        let mut positions: Vec<[f32; 3]> = Vec::<[f32; 3]>::new();
        let mut normals: Vec<[f32; 3]> = Vec::<[f32; 3]>::new();
        let mut colors: Vec<[f32; 3]> = Vec::<[f32; 3]>::new();
        let mut uvs: Vec<(usize, [f32; 2])> = Vec::new();

        if reader.read_positions().is_some() {
            positions = {
                let iter = reader.read_positions().unwrap();
                iter.collect::<Vec<_>>()
            };
        }
        if reader.read_normals().is_some() {
            normals = {
                let iter = reader.read_normals().unwrap();
                iter.collect::<Vec<_>>()
            };
        }
        if reader.read_colors(0).is_some() {
            colors = {
                let iter = reader.read_colors(0).unwrap().into_rgb_f32();
                iter.collect::<Vec<_>>()
            };
        }
        if reader.read_tex_coords(0).is_some() {
            uvs = {
                let iter = reader.read_tex_coords(0).unwrap().into_f32();
                iter.enumerate().collect::<Vec<_>>()
            };
        }

        let mut vertices: Vec<rendering::common::Vertex> = Vec::new();
        for i in 0..positions.len() {
            vertices.push(rendering::common::Vertex::new(
                if positions.len() > 0 {
                    positions[i]
                } else {
                    [0.0, 0.0, 0.0]
                },
                if colors.len() > 0 {
                    colors[i]
                } else {
                    [0.0, 0.0, 0.0]
                },
                if uvs.len() > 0 { uvs[i].1 } else { [0.0, 0.0] },
                if normals.len() > 0 {
                    normals[i]
                } else {
                    [0.0, 0.0, 1.0]
                },
            ));
        }
        let mut indices = {
            let iter = reader.read_indices().unwrap().into_u32();
            iter.collect::<Vec<_>>()
        };

        mesh_vertices.append(&mut vertices);
        mesh_indices.append(&mut indices);

        /*
        log::debug!(
            "Mesh : vertice {}, indices {}",
            mesh_vertices.len(),
            mesh_indices.len()
        );*/
    }

    rendering::common::Mesh {
        _name: mesh.name().unwrap().to_string(),
        vertices: mesh_vertices,
        indices: mesh_indices,
    }
}

// .obj

#[allow(dead_code)]
pub async fn load_obj_single(file_name: &str) -> rendering::common::Mesh {
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

#[allow(dead_code)]
pub async fn load_obj(file_name: &str) -> Vec<rendering::common::Mesh> {
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

    let models: Vec<tobj::Model> = loaded_obj.0;
    let mut out: Vec<rendering::common::Mesh> = Vec::new();

    for model in models.iter() {
        log::debug!(
            "loaded {} : vertices {}, indices {}",
            file_name,
            model.mesh.positions.len() / 3,
            model.mesh.indices.len()
        );

        if model.mesh.positions.len() == 0 {
            continue;
        }

        out.push(rendering::common::Mesh {
            _name: model.name.clone(),
            vertices: rendering::common::create_vertices_from_obj(&model, true),
            indices: rendering::common::create_indices_from_obj(&model, true),
        });
    }

    return out;
}
