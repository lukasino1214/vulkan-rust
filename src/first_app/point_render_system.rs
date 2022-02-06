use super::lve_device::*;
use super::lve_game_object::*;
use super::lve_pipeline::*;
use super::lve_frame_info::*;
use super::lve_camera::*;

use ash::{vk, Device};

use std::rc::Rc;

extern crate nalgebra as na;

#[repr(align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Align16<T>(pub T);
#[derive(PartialEq)]
struct PointLightPushConstants {
    position: na::Vector4<f32>,
    color: na::Vector4<f32>,
    radius: f32,
}

impl PointLightPushConstants {
    pub unsafe fn as_bytes(&self) -> &[u8] {
        let size_in_bytes = std::mem::size_of::<Self>();
        let size_in_u8 = size_in_bytes / std::mem::size_of::<u8>();
        let start_ptr = self as *const Self as *const u8;
        std::slice::from_raw_parts(start_ptr, size_in_u8)
    }
}


pub struct PointRenderSystem {
    lve_device: Rc<LveDevice>,
    lve_pipeline: LvePipeline,
    pipeline_layout: vk::PipelineLayout,
}

impl PointRenderSystem {
    pub fn new(lve_device: Rc<LveDevice>, render_pass: &vk::RenderPass, global_set_layout: &[ash::vk::DescriptorSetLayout]) -> Self {
        let pipeline_layout = Self::create_pipeline_layout(&lve_device.device, global_set_layout);

        let lve_pipeline = Self::create_pipeline(Rc::clone(&lve_device), render_pass, &pipeline_layout);

        Self {
            lve_device,
            lve_pipeline,
            pipeline_layout,
        }
    }

    fn create_pipeline(
        lve_device: Rc<LveDevice>,
        render_pass: &vk::RenderPass,
        pipeline_layout: &vk::PipelineLayout,
    ) -> LvePipeline {
        assert!(
            pipeline_layout != &vk::PipelineLayout::null(),
            "Cannot create pipeline before pipeline layout"
        );

        let pipeline_config = LvePipeline::default_pipline_config_info();

        LvePipeline::new(
            lve_device,
            "./shaders/point_light.vert",
            "./shaders/point_light.frag",
            pipeline_config,
            render_pass,
            pipeline_layout,
        )
    }

    fn create_pipeline_layout(device: &Device, global_set_layout: &[ash::vk::DescriptorSetLayout]) -> vk::PipelineLayout {
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | ash::vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<PointLightPushConstants>() as u32)
            .build();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(global_set_layout)
            .push_constant_ranges(&[push_constant_range])
            .build();

        unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| log::error!("Unable to create pipeline layout: {}", e))
                .unwrap()
        }
    }

    pub fn update(&self, frame_info: &FrameInfo, ubo: &mut GlobalUbo) {
        let mut light_index = 0;

        for game_obj in frame_info.game_objects.iter() {
            match &game_obj.point_light {
                Some(PointLightComponent) => {
                    ubo.point_lights[light_index].position = na::vector![game_obj.transform.translation.x, game_obj.transform.translation.y, game_obj.transform.translation.z, 1.0];
                    ubo.point_lights[light_index].color = na::vector![game_obj.color.x, game_obj.color.y, game_obj.color.z, game_obj.point_light.as_ref().unwrap().light_intensity];
                    ubo.num_lights += 1;
                    light_index += 1;
                }
                _ => {},
            }
        }
    }

    pub fn render(
        &mut self,
        frame_info: &FrameInfo
    ) {
        unsafe { 
            self.lve_pipeline.bind(&self.lve_device.device, frame_info.command_buffer);
            self.lve_device.device.cmd_bind_descriptor_sets(
                frame_info.command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[frame_info.global_descriptor_set],
                &[],
            );

            for game_obj in frame_info.game_objects.iter() {
                match &game_obj.point_light {
                    Some(point_light) => {
                        let push = PointLightPushConstants {
                            position: na::vector![game_obj.transform.translation.x, game_obj.transform.translation.y, game_obj.transform.translation.z, 1.0],
                            color: na::vector![game_obj.color.x, game_obj.color.y, game_obj.color.z, point_light.light_intensity],
                            radius: game_obj.transform.scale.x,
                        };

                        let push_ptr = push.as_bytes();

                        self.lve_device.device.cmd_push_constants(
                            frame_info.command_buffer,
                            self.pipeline_layout,
                            ash::vk::ShaderStageFlags::VERTEX | ash::vk::ShaderStageFlags::FRAGMENT,
                            0,
                            push_ptr,
                        );

                        self.lve_device.device.cmd_draw(frame_info.command_buffer, 6, 1,0,0);
                    },
                    None => { },
                }
            }

            //self.lve_device.device.cmd_draw(frame_info.command_buffer, 6, 1, 0, 0);
        };

        // let projection_view = frame_info.camera.projection_matrix * frame_info.camera.view_matrix;

        // for game_obj in game_objects.iter_mut() {

        //     /*let push = SimplePushConstantData {
        //         model_matrix: Align16(game_obj.transform.mat4()),
        //         normal_matrix: Align16(game_obj.transform.normal_matrix())
        //     };*/

        //     unsafe {
        //         /*let push_ptr = push.as_bytes();

        //         self.lve_device.device.cmd_push_constants(
        //             frame_info.command_buffer,
        //             self.pipeline_layout,
        //             vk::ShaderStageFlags::VERTEX,
        //             0,
        //             push_ptr,
        //         );*/

        //         game_obj.model.bind(&self.lve_device.device, frame_info.command_buffer);
        //         game_obj.model.draw(&self.lve_device.device, frame_info.command_buffer);
        //     }
        // }
    }
}

impl Drop for PointRenderSystem {
    fn drop(&mut self) {
        log::debug!("Dropping SimpleRenderSystem");

        unsafe {
            self.lve_device.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
