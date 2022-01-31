use super::lve_device::*;

use ash::{vk, Device};

use std::mem::size_of;
use std::rc::Rc;
use std::str::FromStr;

extern crate nalgebra as na;

type Pos = na::Vector3<f32>;
type Color = na::Vector3<f32>;

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Pos,
    pub color: Color,
}

pub struct Builder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}

impl Builder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }
}

impl Vertex {
    pub fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        let vertex_size = size_of::<Vertex>() as u32;

        vec![vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(vertex_size)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    pub fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(size_of::<Pos>() as u32) // Using size of the position field
                .build(),
        ]
    }
}

pub struct LveModel {
    lve_device: Rc<LveDevice>,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_count: u32,
    has_index_buffer: bool,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    index_count: u32,
    name: String,
}

impl LveModel {
    pub fn new(lve_device: Rc<LveDevice>, builder: &Builder, name: &str) -> Rc<Self> {
        let (vertex_buffer, vertex_buffer_memory, vertex_count) =
            Self::create_vertex_buffers(&lve_device, &builder.vertices);
        let (has_index_buffer, index_buffer, index_buffer_memory, index_count) =
            Self::create_index_buffers(&lve_device, &builder.indices);
        Rc::new(Self {
            lve_device,
            vertex_buffer,
            vertex_buffer_memory,
            vertex_count,
            has_index_buffer,
            index_buffer,
            index_buffer_memory,
            index_count,
            name: String::from_str(name).unwrap(),
        })
    }

    pub fn new_null(lve_device: Rc<LveDevice>, name: &str) -> Rc<Self> {
        Rc::new(Self {
            lve_device,
            vertex_buffer: vk::Buffer::null(),
            vertex_buffer_memory: vk::DeviceMemory::null(),
            vertex_count: 0,
            has_index_buffer: false,
            index_buffer: vk::Buffer::null(),
            index_buffer_memory: vk::DeviceMemory::null(),
            index_count: 0,
            name: String::from_str(name).unwrap(),
        })
    }

    pub unsafe fn draw(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        if self.has_index_buffer {
            device.cmd_draw(command_buffer, self.index_count, 1, 0, 0);
        } else {
            device.cmd_draw(command_buffer, self.vertex_count, 1, 0, 0);
        }
    }

    pub unsafe fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        let buffers = [self.vertex_buffer];
        let offsets = [0 as u64];
        device.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);

        if self.has_index_buffer {
            device.cmd_bind_index_buffer(command_buffer, self.index_buffer, 0, vk::IndexType::UINT32);
        }
    }

    fn create_vertex_buffers(lve_device: &Rc<LveDevice>, vertices: &Vec<Vertex>) -> (vk::Buffer, vk::DeviceMemory, u32) {
        let vertex_count = vertices.len();
        assert!(vertex_count >= 3, "Vertex count must be at least 3");

        let buffer_size: vk::DeviceSize = (size_of::<Vertex>() * vertex_count) as u64;

        let (vertex_buffer, vertex_buffer_memory) = lve_device.create_buffer(
            buffer_size,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, // Accessible from the host (cpu)
        );

        unsafe {
            let data_ptr = lve_device
                .device
                .map_memory(
                    vertex_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| log::error!("Unable to map vertex buffer memory: {}", e))
                .unwrap();

            let mut align =
                ash::util::Align::new(data_ptr, std::mem::align_of::<u32>() as u64, buffer_size);

            align.copy_from_slice(vertices);

            lve_device.device.unmap_memory(vertex_buffer_memory);
        };

        (vertex_buffer, vertex_buffer_memory, vertex_count as u32)
    }

    fn create_index_buffers(lve_device: &Rc<LveDevice>, indices: &Vec<u32>) -> (bool, vk::Buffer, vk::DeviceMemory, u32) {
        let index_count = indices.len();
        let has_index_buffer = index_count > 0;

        if !has_index_buffer {
            let index_buffer = vk::Buffer::null();
            let index_buffer_memory = vk::DeviceMemory::null();
            return (has_index_buffer, index_buffer, index_buffer_memory, 0);
        }

        let buffer_size: vk::DeviceSize = (size_of::<u32>() * index_count) as u64;

        let (index_buffer, index_buffer_memory) = lve_device.create_buffer(
            buffer_size,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, // Accessible from the host (cpu)
        );

        unsafe {
            let data_ptr = lve_device
                .device
                .map_memory(
                    index_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| log::error!("Unable to map vertex buffer memory: {}", e))
                .unwrap();

            let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<u32>() as u64, buffer_size);

            align.copy_from_slice(indices);

            //data_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());


            /*let mut index_slice = ash::util::Align::new(
                data_ptr,
                std::mem::align_of::<u32>() as u64,
                buffer_size,
            );
            index_slice.copy_from_slice(&indices);*/

            lve_device.device.unmap_memory(index_buffer_memory);
        };

        (has_index_buffer, index_buffer, index_buffer_memory, index_count as u32)
    }
}

impl Drop for LveModel {
    fn drop(&mut self) {
        log::debug!("Dropping Model: {}", self.name);
        unsafe {
            self.lve_device.device.destroy_buffer(self.vertex_buffer, None);
            self.lve_device.device.free_memory(self.vertex_buffer_memory, None);

            if self.has_index_buffer {
                self.lve_device.device.destroy_buffer(self.index_buffer, None);
                self.lve_device.device.free_memory(self.index_buffer_memory, None);
            }
        }
    }
}
