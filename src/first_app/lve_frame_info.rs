extern crate nalgebra as na;
use super::lve_camera::*;

#[derive(PartialEq)]
pub struct Align16<T>(pub T);
#[derive(PartialEq)]
pub struct GlobalUbo {
    pub projection_matrix: na::Matrix4<f32>,
    pub view_matrix: na::Matrix4<f32>,
    pub ambient_light_color: na::Vector4<f32>,
    pub light_position: na::Vector3<f32>,
    pub padding: u32,
    pub light_color: na::Vector4<f32>
}

pub struct FrameInfo {
    pub frame_index: usize,
    pub frame_time: f32,
    pub command_buffer: ash::vk::CommandBuffer,
    pub camera: LveCamera,
    pub global_descriptor_set: ash::vk::DescriptorSet
}