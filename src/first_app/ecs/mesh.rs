use crate::first_app::vulkan::lve_descriptor_set::*;
use crate::first_app::vulkan::lve_device::*;
use crate::first_app::vulkan::lve_buffer::*;
use crate::first_app::vulkan::lve_image::*;

use ash::{vk, Device};

use std::mem::size_of;
use std::rc::Rc;

use nalgebra as na;

pub struct MeshTextures {
    pub base_color: LveImage,
    pub metallic_roughness: LveImage,
    pub normal: LveImage,
    pub occlusion: LveImage,
    pub emissive: LveImage,
}

impl MeshTextures {
    pub fn new(lve_device: Rc<LveDevice>) -> Self {
        Self {
            base_color: LveImage::default(lve_device.clone()),
            metallic_roughness: LveImage::default(lve_device.clone()),
            normal: LveImage::default(lve_device.clone()),
            occlusion: LveImage::default(lve_device.clone()),
            emissive: LveImage::default(lve_device.clone()),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct MeshUniforms {
    pub base_color: na::Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: na::Vector3<f32>,
}

impl MeshUniforms {
    pub fn new() -> Self {
        Self {
            base_color: na::vector![1.0, 1.0, 1.0], // Maybe add alpha channel
            metallic: 1.0,
            roughness: 1.0,
            emissive: na::vector![1.0, 1.0, 1.0],
        }
    }
}



#[derive(Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: na::Vector3<f32>,
    pub color: na::Vector3<f32>,
    pub normal: na::Vector3<f32>,
    pub tangent: na::Vector4<f32>,
    pub tex_coord: na::Vector2<f32>,
}

impl Vertex {
    #[allow(dead_code)]
    pub fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        let vertex_size = size_of::<Vertex>() as u32;

        vec![vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(vertex_size)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    #[allow(dead_code)]
    pub fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, position) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, color) as u32) // Using size of the position field
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, normal) as u32) // Using size of the position field
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, tangent) as u32) // Using size of the position field
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(4)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, tex_coord) as u32) // Using size of the position field
                .build(),
        ]
    }
}

#[allow(dead_code)]
pub struct Mesh {
    vertex_buffer: LveBuffer<Vertex>,
    vertex_count: u32,
    has_index_buffer: bool,
    index_buffer: LveBuffer<u32>,
    index_count: u32,
    uniform_buffer: LveBuffer<MeshUniforms>,
    textures: MeshTextures,
    pub descriptor_set: ash::vk::DescriptorSet,
    //descriptor_layout: Rc<LveDescriptorSetLayout>
}

impl Mesh {
    pub fn new(lve_device: &Rc<LveDevice>, vertices: Vec<Vertex>, indices: Vec<u32>, textures: MeshTextures, uniforms: MeshUniforms, global_pool: Rc<LveDescriptorPool>) -> Rc<Self> {
        let (vertex_buffer, vertex_count) = Self::create_vertex_buffers(&lve_device, &vertices);
        let (has_index_buffer, index_buffer, index_count) = Self::create_index_buffers(&lve_device, &indices);

        let mut uniform_buffer = LveBuffer::new(
            Rc::clone(&lve_device),
            1,
            ash::vk::BufferUsageFlags::UNIFORM_BUFFER,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
        );

        uniform_buffer.map(0);

        uniform_buffer.write_to_buffer(&[uniforms]);

        let descriptor_layout = LveDescriptorSetLayout::new(Rc::clone(&lve_device))
            .add_binding(0, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(1, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(2, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(3, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(4, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(5, ash::vk::DescriptorType::UNIFORM_BUFFER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .build().unwrap();

        let descriptor_set = LveDescriptorSetWriter::new(descriptor_layout, global_pool.clone())
            .write_image(0, &[textures.base_color.image_info])
            .write_image(1, &[textures.metallic_roughness.image_info])
            .write_image(2, &[textures.normal.image_info])
            .write_image(3, &[textures.occlusion.image_info])
            .write_image(4, &[textures.emissive.image_info])
            .write_to_buffer(5, &[uniform_buffer.descriptor_info()])
            .build().unwrap();

        
        Rc::new(Self {
            vertex_buffer,
            vertex_count,
            has_index_buffer,
            index_buffer,
            index_count,
            uniform_buffer,
            textures,
            descriptor_set,
            //descriptor_layout
        })
    }

    /*#[allow(dead_code)]
    pub fn new_null(lve_device: Rc<LveDevice>) -> Rc<Self> {
        let vertex_buffer = LveBuffer::null(Rc::clone(&lve_device));
        let index_buffer = LveBuffer::null(Rc::clone(&lve_device));
        Rc::new(Self {
            vertex_buffer,
            vertex_count: 0,
            has_index_buffer: false,
            index_buffer,
            index_count: 0,
            image_descriptor_set: ash::vk::DescriptorSet::null(),
            image: ash::vk::Image::null()
        })
    }*/

    pub unsafe fn draw(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        if self.has_index_buffer {
            device.cmd_draw_indexed(command_buffer, self.index_count, 1, 0, 0, 0);
        } else {
            device.cmd_draw(command_buffer, self.vertex_count, 1, 0, 0);
        }
    }

    pub unsafe fn bind(&self, command_buffer: vk::CommandBuffer) {
        self.vertex_buffer.bind_vertex(command_buffer);

        if self.has_index_buffer {
            //device.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT32);
            self.index_buffer.bind_index(command_buffer, vk::IndexType::UINT32);
        }
    }

    fn create_vertex_buffers(lve_device: &Rc<LveDevice>, vertices: &Vec<Vertex>) -> (LveBuffer<Vertex>, u32) {
        let vertex_count = vertices.len();
        assert!(vertex_count >= 3, "Vertex count must be at least 3");

        let buffer_size: vk::DeviceSize = (size_of::<Vertex>() * vertex_count) as u64;

        let mut staging_buffer = LveBuffer::new(
            lve_device.clone(),
            vertex_count,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        staging_buffer.map(0);
        staging_buffer.write_to_buffer(vertices);

        let vertex_buffer = LveBuffer::new(
            lve_device.clone(),
            vertex_count,
            ash::vk::BufferUsageFlags::VERTEX_BUFFER | ash::vk::BufferUsageFlags::TRANSFER_DST,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        lve_device.copy_buffer(staging_buffer.buffer, vertex_buffer.buffer, buffer_size);

        (vertex_buffer, vertex_count as u32)
    }

    fn create_index_buffers(lve_device: &Rc<LveDevice>, indices: &Vec<u32>) -> (bool, LveBuffer<u32>, u32) {
        let index_count = indices.len();
        let has_index_buffer = index_count > 0;

        if !has_index_buffer {
            let index_buffer = LveBuffer::null(lve_device.clone());
            //let index_buffer_memory = vk::DeviceMemory::null();
            return (has_index_buffer, index_buffer, 0);
        }

        let buffer_size: vk::DeviceSize = (size_of::<u32>() * index_count) as u64;

        let mut staging_buffer = LveBuffer::new(
            lve_device.clone(),
            index_count,
            ash::vk::BufferUsageFlags::TRANSFER_SRC,
            ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        staging_buffer.map(0);
        staging_buffer.write_to_buffer(indices);

        let index_buffer = LveBuffer::new(
            lve_device.clone(),
            index_count,
            ash::vk::BufferUsageFlags::INDEX_BUFFER | ash::vk::BufferUsageFlags::TRANSFER_DST,
            ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        lve_device.copy_buffer(staging_buffer.buffer, index_buffer.buffer, buffer_size);

        (has_index_buffer, index_buffer, index_count as u32)
    }
}