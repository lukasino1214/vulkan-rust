extern crate nalgebra as na;

use super::{lve_camera::*, lve_game_object::LveGameObject};

pub const MAX_LIGHTS: usize = 10;

#[repr(align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Align16<T>(pub T);
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub struct PointLight {
    pub position: na::Vector4<f32>,
    pub color: na::Vector4<f32>,
}

#[derive(PartialEq)]
pub struct GlobalUbo {
    pub projection_matrix: Align16<na::Matrix4<f32>>,
    pub view_matrix: Align16<na::Matrix4<f32>>,
    pub camera_position: Align16<na::Vector3<f32>>,
    pub ambient_light_color: Align16<na::Vector4<f32>>,
    pub point_lights: [PointLight; MAX_LIGHTS],
    pub num_lights: u32,
}

pub struct FrameInfo<'a> {
    pub frame_index: usize,
    pub frame_time: f32,
    pub command_buffer: ash::vk::CommandBuffer,
    pub camera: LveCamera,
    pub global_descriptor_set: ash::vk::DescriptorSet,
    pub image_descriptor_set: ash::vk::DescriptorSet,
    pub game_objects: &'a Vec<LveGameObject>
}