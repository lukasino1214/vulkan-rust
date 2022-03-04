use super::mesh::*;

use crate::first_app::vulkan::lve_descriptor_set::*;
use crate::first_app::vulkan::lve_device::*;
use crate::first_app::vulkan::lve_frame_info::*;
use crate::first_app::vulkan::lve_image::*;

use std::rc::Rc;
use std::str::FromStr;
use std::time::{Instant};

use nalgebra as na;

#[allow(dead_code)]
pub struct Model {
    sub_meshes: Vec<Rc<Mesh>>, // in gltf it is primitive
    file_path: String
}

impl Model {
    pub fn new(lve_device: &Rc<LveDevice>, path: &str, global_pool: Rc<LveDescriptorPool>) -> Self{
        let mut sub_meshes = Vec::new();

        let relative_path = std::path::Path::new(path).parent().unwrap().to_str().unwrap();

        let start = Instant::now();

        println!("Loading {}", path);
        let (document, buffers, _) = gltf::import(path).unwrap();
        for mesh in document.meshes() {
            let primitives: Vec<gltf::Primitive> = mesh.primitives().collect();
            primitives.iter().for_each(|primitive| {
                // Vectors for mesh
                let mut vertices: Vec<Vertex> = Vec::new();

                // Get reader
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Read data to the iter
                let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
                let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
                let tangents_option = reader.read_tangents();
                //let mut colors_iter = reader.read_colors(0).unwrap();
                let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0).unwrap().into_f32().collect();
                let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

                let mut has_tangents = false;
                let mut tangents: Vec<[f32; 4]> = Vec::new();

                match tangents_option {
                    Some(_) => {
                        tangents = tangents_option.unwrap().collect();
                        has_tangents = true;
                    }

                    None => {}
                }

                let count = positions.len();

                for i in 0..count {
                    // Get vertex informations from vectros
                    let position = positions[i];
                    let normal = normals[i];
                    let mut tangent = [0.0, 0.0, 0.0, 0.0];
                    if has_tangents {
                        tangent = tangents[i];
                    }
                    let tex_coord = tex_coords[i];

                    let vertex = Vertex {
                        position: na::vector![position[0], position[1], position[2]],
                        color: na::vector![1.0, 1.0, 1.0],
                        normal: na::vector![normal[0], normal[1], normal[2]],
                        tangent: na::vector![tangent[0], tangent[1], tangent[2], tangent[3]],
                        tex_coord: na::vector![tex_coord[0], tex_coord[1]],
                    };

                    vertices.push(vertex);
                }

                let mut textures = MeshTextures::new(lve_device.clone());
                let mut uniforms = MeshUniforms::new();

                let albedo_texture = primitive.material().pbr_metallic_roughness().base_color_texture();
                match albedo_texture {
                    Some(_) => {
                        let mut image_path: String = "".to_string();
                        let texture = albedo_texture.unwrap().texture().source().source();
                        if let gltf::image::Source::Uri {uri, ..} = texture {
                            image_path =  relative_path.to_owned() + "/" + uri;
                        }

                        textures.base_color = LveImage::new(lve_device.clone(), image_path.as_str());
                    },
                    None => {
                        let color = primitive.material().pbr_metallic_roughness().base_color_factor();
                        uniforms.base_color = na::vector![color[0], color[1], color[2]];
                    }
                }

                let metallic_roughness_texture = primitive.material().pbr_metallic_roughness().metallic_roughness_texture();
                match metallic_roughness_texture {
                    Some(_) => {
                        let mut image_path: String = "".to_string();
                        let texture = metallic_roughness_texture.unwrap().texture().source().source();
                        if let gltf::image::Source::Uri {uri, ..} = texture {
                            image_path =  relative_path.to_owned() + "/" + uri;
                        }

                        textures.metallic_roughness = LveImage::new(lve_device.clone(), image_path.as_str());
                    },
                    None => {
                        uniforms.metallic = primitive.material().pbr_metallic_roughness().metallic_factor();
                        uniforms.roughness = primitive.material().pbr_metallic_roughness().roughness_factor();
                    }
                }

                let normal_texture = primitive.material().normal_texture();
                match normal_texture {
                    Some(_) => {
                        let mut image_path: String = "".to_string();
                        let texture = normal_texture.unwrap().texture().source().source();
                        if let gltf::image::Source::Uri {uri, ..} = texture {
                            image_path =  relative_path.to_owned() + "/" + uri;
                        }

                        textures.normal = LveImage::new(lve_device.clone(), image_path.as_str());
                    },
                    None => {}
                }

                let occlusion_texture = primitive.material().occlusion_texture();
                match occlusion_texture {
                    Some(_) => {
                        let mut image_path: String = "".to_string();
                        let texture = occlusion_texture.unwrap().texture().source().source();
                        if let gltf::image::Source::Uri {uri, ..} = texture {
                            image_path =  relative_path.to_owned() + "/" + uri;
                        }

                        textures.occlusion = LveImage::new(lve_device.clone(), image_path.as_str());
                    },
                    None => {}
                }

                let emessive_texture = primitive.material().emissive_texture();
                match emessive_texture {
                    Some(_) => {
                        let mut image_path: String = "".to_string();
                        let texture = emessive_texture.unwrap().texture().source().source();
                        if let gltf::image::Source::Uri {uri, ..} = texture {
                            image_path =  relative_path.to_owned() + "/" + uri;
                        }

                        textures.metallic_roughness = LveImage::new(lve_device.clone(), image_path.as_str());
                    },
                    None => {
                        let emissive = primitive.material().emissive_factor();
                        uniforms.emissive = na::vector![emissive[0], emissive[1], emissive[2]];
                    }
                }

                let mesh = Mesh::new(lve_device.clone(), vertices, indices, textures, uniforms, global_pool.clone());

                sub_meshes.push(mesh);
            });
        }
        println!("Loaded {}", path);
        let duration = start.elapsed();

        println!("It took only: {:?}", duration);

        Self {
            sub_meshes,
            file_path: String::from_str(path).unwrap()
        }
    }

    pub fn render(&self, device: &ash::Device, frame_info: &FrameInfo, pipeline_layout: ash::vk::PipelineLayout) {
        for mesh in self.sub_meshes.iter() {
            unsafe {
                device.cmd_bind_descriptor_sets(
                    frame_info.command_buffer,
                    ash::vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &[frame_info.global_descriptor_set, mesh.descriptor_set],
                    &[],
                );

                mesh.bind(frame_info.command_buffer);
                mesh.draw(device, frame_info.command_buffer);
            }
        }
    }
}