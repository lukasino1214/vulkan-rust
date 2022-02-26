use super::mesh::*;

use crate::first_app::vulkan::lve_descriptor_set::*;
use crate::first_app::vulkan::lve_device::*;
use crate::first_app::vulkan::lve_frame_info::*;
use crate::first_app::vulkan::lve_image::*;

use std::rc::Rc;
use std::str::FromStr;

use nalgebra as na;

#[allow(dead_code)]
pub struct Model {
    sub_meshes: Vec<Rc<Mesh>>, // in gltf it is primitive
    file_path: String
}

impl Model {
    pub fn new(lve_device: &Rc<LveDevice>, path: &str, image_set_layout: Rc<LveDescriptorSetLayout>, global_pool: Rc<LveDescriptorPool>) -> Self{
        let mut sub_meshes = Vec::new();

        let relative_path = std::path::Path::new(path).parent().unwrap().to_str().unwrap();

        let (document, buffers, _) = gltf::import(path).unwrap();
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                // Vectors for mesh
                let mut vertices: Vec<Vertex> = Vec::new();
                let mut indices: Vec<u32> = Vec::new();

                // Get reader
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Read data to the iter
                let positions_iter = reader.read_positions().unwrap();
                let normals_iter = reader.read_normals().unwrap();
                //let mut colors_iter = reader.read_colors(0).unwrap();
                let tex_coords_iter = reader.read_tex_coords(0).unwrap().into_f32();
                let indices_iter = reader.read_indices().unwrap();

                // Stupidly convert iter to vec
                for index in indices_iter.into_u32() {
                    indices.push(index);
                }

                let mut positions_vec = Vec::new();
                for vertex_position in positions_iter {
                    positions_vec.push(vertex_position);
                }

                let mut normals_vec = Vec::new();
                for vertex_normal in normals_iter {
                    normals_vec.push(vertex_normal);
                }

                let mut tex_coords_vec = Vec::new();
                for vertex_tex_coord in tex_coords_iter {
                    tex_coords_vec.push(vertex_tex_coord);
                }

                let count = positions_vec.len();

                for i in 0..count {
                    // Get vertex informations from vectros
                    let position = positions_vec[i];
                    let normal = normals_vec[i];
                    let tex_coord = tex_coords_vec[i];

                    let vertex = Vertex {
                        position: na::vector![position[0], position[1], position[2]],
                        color: na::vector![1.0, 1.0, 1.0],
                        normal: na::vector![normal[0], normal[1], normal[2]],
                        tex_coord: na::vector![tex_coord[0], tex_coord[1]],
                    };

                    vertices.push(vertex);
                }

                let mut image_path: String = "".to_string();
                let albedo_texture = primitive.material().pbr_metallic_roughness().base_color_texture().unwrap().texture().source().source();
                if let gltf::image::Source::Uri {uri, ..} = albedo_texture {
                    image_path =  relative_path.to_owned() + "/" + uri;
                }

                let image = LveImage::new(Rc::clone(&lve_device), image_path.as_str());


                let mesh = Mesh::new(lve_device, vertices, indices, image, image_set_layout.clone(), global_pool.clone());

                sub_meshes.push(mesh);
            }
        }
        println!("Loaded {}", path);

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
                    &[frame_info.global_descriptor_set, mesh.image_descriptor_set],
                    &[],
                );

                mesh.bind(frame_info.command_buffer);
                mesh.draw(device, frame_info.command_buffer);
            }
        }
    }
}