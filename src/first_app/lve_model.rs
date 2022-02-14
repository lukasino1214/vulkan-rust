use super::lve_device::*;
use super::lve_buffer::*;

use ash::{vk, Device};

use std::mem::size_of;
use std::rc::Rc;

extern crate nalgebra as na;

type Pos = na::Vector3<f32>;
type Color = na::Vector3<f32>;
type Normal = na::Vector3<f32>;
type UV = na::Vector2<f32>;

#[derive(Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: Pos,
    pub color: Color,
    pub normal: Normal,
    pub uv: UV
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

    pub fn load_from_file(&mut self, file_path: &str) {
        let (models, _) = tobj::load_obj(
            file_path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        ).unwrap();

        for model in models.iter() {
            let mesh = &models[0].mesh;

            let positions = mesh.positions.as_slice();
            let colors = mesh.vertex_color.as_slice();
            let normals = mesh.normals.as_slice();
            let coords = mesh.texcoords.as_slice();

            let vertex_count = mesh.positions.len() / 3;

            let mut vertices = Vec::with_capacity(vertex_count);
            for i in 0..vertex_count {
                let x = positions[3 * i + 0];
                let y = positions[3 * i + 1];
                let z = positions[3 * i + 2];

                let color_x;
                let color_y;
                let color_z;

                //aint working

                let color_index = 3 * i + 2;
                if color_index < colors.len() {
                    color_x = colors[3 * i - 2];
                    color_y = colors[3 * i - 1];
                    color_z = colors[3 * i - 0];
                } else {
                    color_x = 1.0;
                    color_y = 1.0;
                    color_z = 1.0;
                }

                let normal_x = normals[3 * i + 0];
                let normal_y = normals[3 * i + 1];
                let normal_z = normals[3 * i + 2];

                let u = coords[2 * i + 0];
                let v = coords[2 * i + 1];

                let vertex = Vertex {
                    position: na::vector!(x, y, z),
                    color: na::vector!(color_x, color_y, color_z),
                    normal: na::vector!(normal_x, normal_y, normal_z),
                    uv: na::vector!(u, v),
                };

                vertices.push(vertex);
            }

            self.vertices.append(&mut vertices);
            self.indices.append(&mut mesh.indices.clone());
        }
    }

        /*let mesh = &models[0].mesh;

        let positions = mesh.positions.as_slice();
        let colors = mesh.vertex_color.as_slice();
        let normals = mesh.normals.as_slice();
        let coords = mesh.texcoords.as_slice();

        let vertex_count = mesh.positions.len() / 3;

        let mut vertices = Vec::with_capacity(vertex_count);
        for i in 0..vertex_count {
            let x = positions[3 * i + 0];
            let y = positions[3 * i + 1];
            let z = positions[3 * i + 2];

            let color_x;
            let color_y;
            let color_z;

            //aint working

            let color_index = 3 * i + 2;
            if color_index < colors.len() {
                color_x = colors[3 * i - 2];
                color_y = colors[3 * i - 1];
                color_z = colors[3 * i - 0];
            } else {
                color_x = 1.0;
                color_y = 1.0;
                color_z = 1.0;
            }

            let normal_x = normals[3 * i + 0];
            let normal_y = normals[3 * i + 1];
            let normal_z = normals[3 * i + 2];

            let u = coords[2 * i + 0];
            let v = coords[2 * i + 1];

            let vertex = Vertex {
                position: na::vector!(x, y, z),
                color: na::vector!(color_x, color_y, color_z),
                normal: na::vector!(normal_x, normal_y, normal_z),
                uv: na::vector!(u, v),
            };

            vertices.push(vertex);
        }

        self.vertices = vertices;
        self.indices = mesh.indices.clone();
    }*/
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
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(Vertex, uv) as u32) // Using size of the position field
                .build(),
        ]
    }
}

pub struct LveModel {
    vertex_buffer: LveBuffer<Vertex>,
    vertex_count: u32,
    has_index_buffer: bool,
    index_buffer: LveBuffer<u32>,
    index_count: u32,
}

impl LveModel {
    pub fn new(lve_device: Rc<LveDevice>, builder: &Builder) -> Rc<Self> {
        let (vertex_buffer, vertex_count) = Self::create_vertex_buffers(&lve_device, &builder.vertices);
        let (has_index_buffer, index_buffer, index_count) = Self::create_index_buffers(&lve_device, &builder.indices);
        
        Rc::new(Self {
            vertex_buffer,
            vertex_count,
            has_index_buffer,
            index_buffer,
            index_count,
        })
    }

    pub fn new_from_file(lve_device: Rc<LveDevice>, file_path: &str) -> Rc<Self> {
        let mut builder = Builder::new();
        builder.load_from_file(file_path);

        LveModel::new(lve_device, &builder)
    }

    #[allow(dead_code)]
    pub fn new_null(lve_device: Rc<LveDevice>) -> Rc<Self> {
        let vertex_buffer = LveBuffer::null(Rc::clone(&lve_device));
        let index_buffer = LveBuffer::null(Rc::clone(&lve_device));
        Rc::new(Self {
            vertex_buffer,
            vertex_count: 0,
            has_index_buffer: false,
            index_buffer,
            index_count: 0,
        })
    }

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