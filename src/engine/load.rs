use crate::engine;
use crate::rendering;
use image::GenericImageView;

// Utility

#[allow(dead_code)]
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
#[allow(dead_code)]
pub async fn file_status(file_name: &str) -> u16 {
    let url = format_url(file_name);
    let status = reqwest::get(url).await.unwrap().status();
    let code = status.as_u16();

    return code;
}
#[allow(dead_code)]
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
#[allow(dead_code)]
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
            //log::debug!("Load {} byte from {}", data.len(), file_name);
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
#[allow(dead_code)]
async fn extract_texture_data(file_name: &String) -> (Vec<u8>, [u32; 2]) {
    let mut out_data: Vec<u8> = Vec::new();
    let mut out_size: [u32; 2] = [1, 1];

    // assume .png or .bmp only
    let png_path: &String = file_name;
    let rgba_path: String = png_path.replace(".png", ".rgba");
    //let bmp_path: String = png_path.replace(".png", ".bmp");

    // this check is slow, so assume .rgba is exist
    /*
    let is_exist_rgba: bool = !load_string(&rgba_path.as_str())
        .await
        .unwrap()
        .starts_with("<!DOCTYPE html>");
    */
    let is_exist_rgba = true;

    // Load .rgba - ref : bin/image_convert.rs
    if is_exist_rgba {
        let texture_data = load_binary(&rgba_path.as_str())
            .await
            .expect("Failed to load texture");

        let data_width: u32 = load_4byte_to_u32(&texture_data[0..4]);
        let data_height: u32 = load_4byte_to_u32(&texture_data[4..8]);

        out_data = texture_data[8..texture_data.len()].to_vec();
        out_size = [data_width, data_height];
    }
    // Load .bmp
    /*
    else if is_exist_rgba{
        let texture_data = load_binary(&bmp_path.as_str())
            .await
            .expect("Failed to load texture");

        let info_header_ofset: usize = 14;
        let data_width: u32 =
            load_4byte_to_u32(&texture_data[(info_header_ofset + 4)..(info_header_ofset + 8)]);
        let data_height: u32 =
            load_4byte_to_u32(&texture_data[(info_header_ofset + 8)..(info_header_ofset + 12)]);

        let data_offset: usize = load_4byte_to_u32(&texture_data[10..14]) as usize;
        let data_len = (data_width * data_height * 4) as usize;

        out_data = texture_data[data_offset..(data_offset + data_len)].to_vec();
        out_size = [data_width, data_height];

        log::debug!("{}", out_data.len());

        /* too slow
        let texture_image =
        image::load_from_memory_with_format(&texture_data, image::ImageFormat::Bmp);
        if texture_image.is_ok() {
            log::debug!("loaded");
            let texture_image_unwrap = texture_image.unwrap();
            out_data = texture_image_unwrap.to_rgba8().to_vec();
            out_size = [
                texture_image_unwrap.dimensions().0,
                texture_image_unwrap.dimensions().1,
            ];
        }
        */
    }*/
    // Load .png slower
    else {
        let texture_data = load_binary(&png_path.as_str())
            .await
            .expect("Failed to load texture");
        let texture_image =
            image::load_from_memory_with_format(&texture_data, image::ImageFormat::Png);

        if texture_image.is_ok() {
            let texture_image_unwrap = texture_image.unwrap();
            out_data = texture_image_unwrap.to_rgba8().to_vec();
            out_size = [
                texture_image_unwrap.dimensions().0,
                texture_image_unwrap.dimensions().1,
            ];
        }
    }

    return (out_data, out_size);
}
#[allow(dead_code)]
fn load_4byte_to_u32(bytes: &[u8]) -> u32 {
    let out_value: u32 = ((bytes[0] as u32) << 24)
        + ((bytes[1] as u32) << 16)
        + ((bytes[2] as u32) << 8)
        + ((bytes[3] as u32) << 0);

    return out_value;
}

// Load .gltf

pub async fn load_gltf_scene(
    file_name: &str,
) -> (
    Vec<engine::scene::SceneObject>,
    Vec<engine::scene::SceneMaterial>,
) {
    let gltf_text = load_string(file_name)
        .await
        .expect("Failed to parse .gltf file path string");
    let gltf_cursor: std::io::Cursor<String> = std::io::Cursor::new(gltf_text);
    let gltf_reader: std::io::BufReader<_> = std::io::BufReader::new(gltf_cursor);

    let gltf: gltf::Gltf = gltf::Gltf::from_reader(gltf_reader).expect("Failed to read .gltf file");
    let mut buffer_data: Vec<Vec<u8>> = Vec::new();

    let slash_num: usize = file_name.rfind("/").unwrap() + 1;
    let folder_path = file_name.split_at(slash_num).0;

    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                //if let Some(blob) = gltf.blob.as_deref() {
                //    buffer_data.push(blob.into());
                //    log::debug!("Found a bin, saving");
                //};
            }
            gltf::buffer::Source::Uri(uri) => {
                let binary_path = folder_path.to_string() + uri;
                let bin = load_binary(&binary_path.as_str())
                    .await
                    .expect("Failed to load binary");
                buffer_data.push(bin);
            }
        }
    }

    let mut out_objects: Vec<engine::scene::SceneObject> = Vec::new();
    let mut out_materials: Vec<engine::scene::SceneMaterial> = Vec::new();
    let mut num_node: u32 = 0;
    let mut num_verts: u32 = 0;
    let mut num_indices: u32 = 0;

    // Create scene object from meshes
    for node in gltf.nodes() {
        //log::debug!("Node : {}", node.name().unwrap());

        let mut mesh: Option<rendering::common::Mesh> = None;
        if node.mesh().is_some() {
            mesh = Some(get_gltf_mesh_from_node(&node, &buffer_data));
        }

        // Debug only
        num_node += 1;
        if mesh.is_some() {
            num_verts += mesh.as_ref().unwrap().vertices.len() as u32;
            num_indices += mesh.as_ref().unwrap().indices.len() as u32;
        }

        let mut scene_object = engine::scene::SceneObject {
            _name: Some(node.name().unwrap().to_string()),
            shading_type: 44,
            world_transform: node.transform().matrix(),
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

        out_objects.push(scene_object);
    }

    // Build parent tree
    let mut parent_vec: Vec<Option<u32>> = vec![None; out_objects.len()];
    {
        for object in out_objects.iter() {
            for child in object.child_index.iter() {
                let inner = parent_vec.get_mut(*child as usize).unwrap();
                inner.replace(object.index);
            }
        }
        for i in 0..parent_vec.len() {
            let parent = parent_vec.get(i).unwrap();
            if parent.is_some() {
                let object = out_objects.get_mut(i).unwrap();
                object.parent_index = Some(parent.unwrap());
            }
        }
    }

    // Convert object local matrix to world matrix
    let mut matrix_vec: Vec<[[f32; 4]; 4]> = Vec::with_capacity(out_objects.len());
    for object in &out_objects {
        let mut model_matrix = glam::Mat4::from_cols_array_2d(&object.world_transform);
        if object.parent_index.is_some() {
            let mut parent_index = *object.parent_index.as_ref().unwrap();
            loop {
                model_matrix = glam::Mat4::from_cols_array_2d(
                    &out_objects
                        .get(parent_index as usize)
                        .unwrap()
                        .world_transform,
                ) * model_matrix;

                let parent_option = out_objects.get(parent_index as usize).unwrap().parent_index;

                if parent_option.is_some() {
                    parent_index = parent_option.unwrap();
                    continue;
                }
                break;
            }
        }
        matrix_vec.push(model_matrix.to_cols_array_2d());
    }
    for i in 0..matrix_vec.len() {
        out_objects.get_mut(i).unwrap().world_transform = matrix_vec[i];
    }

    // Load materials
    for material in gltf.materials() {
        let pbr = material.pbr_metallic_roughness();
        let mut base_color_texture_data: Vec<u8> = Vec::new();
        let mut base_color_texture_size: [u32; 2] = [1, 1];

        if pbr.base_color_texture().is_some() {
            let texture_source = &pbr
                .base_color_texture()
                .map(|tex| tex.texture().source().source())
                .expect("texture");

            match texture_source {
                gltf::image::Source::View { view, mime_type: _ } => {
                    // embedded data is yet
                    base_color_texture_data = buffer_data[view.buffer().index()].clone();
                }
                gltf::image::Source::Uri { uri, mime_type: _ } => {
                    // from url
                    let texture_path = folder_path.to_string() + uri;
                    (base_color_texture_data, base_color_texture_size) =
                        extract_texture_data(&texture_path).await;
                }
            };
        }

        if base_color_texture_data.is_empty() {
            base_color_texture_data = [255, 0, 255, 255].to_vec();
        }

        let scene_material = engine::scene::SceneMaterial {
            _name: Some(material.name().unwrap().to_string()),
            base_color_texture_dat: base_color_texture_data,
            base_color_texture_size: base_color_texture_size,
        };

        out_materials.push(scene_material);
    }

    log::debug!(
        "\n {} \n nodes : {}\n verts : {},\n tris  : {},\n mat   : {}",
        &file_name,
        num_node,
        num_verts,
        num_indices / 3,
        out_materials.len()
    );

    return (out_objects, out_materials);
}

fn get_gltf_mesh_from_node(
    node: &gltf::Node<'_>,
    buffer_data: &Vec<Vec<u8>>,
) -> rendering::common::Mesh {
    let mesh: gltf::Mesh<'_> = node.mesh().expect("Got mesh");

    let mut mesh_vertices: Vec<rendering::common::Vertex> = Vec::new();
    let mut mesh_indices: Vec<u32> = Vec::new();
    let mut mesh_material: Option<u32> = None;

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
        mesh_material = if primitive.material().index().is_some() {
            Some(primitive.material().index().unwrap() as u32)
        } else {
            mesh_material
        };

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
        material: mesh_material,
    }
}

// Load .obj

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
        material: None,
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
            material: None,
        });
    }

    return out;
}
