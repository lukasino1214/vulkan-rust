use ash::vk;
use winit::window::Window;
use crate::first_app::vulkan::{lve_device::*, lve_frame_info::FrameInfo};

use std::rc::Rc;

pub struct FrameBufferAttachment {
    image: vk::Image,
    image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    format: vk::Format
}

pub struct Framebuffer {
    width: u32,
    height: u32,
    framebuffer: vk::Framebuffer,
    pub position: FrameBufferAttachment,
    pub normal: FrameBufferAttachment,
    pub albedo: FrameBufferAttachment,
    pub metallic_roughness: FrameBufferAttachment,
    pub depth: FrameBufferAttachment,
    render_pass: vk::RenderPass
}

pub struct DefferedRenderingSystem {
    lve_device: Rc<LveDevice>,
    pub things: Framebuffer,
    pub sampler: vk::Sampler
}

impl DefferedRenderingSystem {
    pub fn new(lve_device: Rc<LveDevice>, width: u32, height: u32) -> Self {
        let position = Self::create_attachment(&lve_device, width, height, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::COLOR_ATTACHMENT);
        let normal = Self::create_attachment(&lve_device, width, height, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::COLOR_ATTACHMENT);
        let albedo = Self::create_attachment(&lve_device, width, height, vk::Format::R8G8B8A8_SRGB, vk::ImageUsageFlags::COLOR_ATTACHMENT);
        let metallic_roughness = Self::create_attachment(&lve_device, width, height, vk::Format::R8G8B8A8_SRGB, vk::ImageUsageFlags::COLOR_ATTACHMENT);
        
        let candidates = vec![
            //vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];
        let valid_depth = lve_device.find_supported_format(
            &candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );

        let depth = Self::create_attachment(&lve_device, width, height, valid_depth, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT);

        let attachment_desc_1 = vk::AttachmentDescription::builder()
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .format(position.format)
            .build();

        let attachment_desc_2 = vk::AttachmentDescription::builder()
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .format(normal.format)
            .build();

        let attachment_desc_3 = vk::AttachmentDescription::builder()
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .format(albedo.format)
            .build();

        let attachment_desc_4 = vk::AttachmentDescription::builder()
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .format(metallic_roughness.format)
            .build();

        let attachment_desc_5 = vk::AttachmentDescription::builder()
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .format(depth.format)
            .build();

        let attachment_descs = [attachment_desc_1, attachment_desc_2, attachment_desc_3, attachment_desc_4, attachment_desc_5];

        let color_attachment_ref_1 = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
        
        let color_attachment_ref_2 = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let color_attachment_ref_3 = vk::AttachmentReference::builder()
            .attachment(2)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let color_attachment_ref_4 = vk::AttachmentReference::builder()
            .attachment(3)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let attachment_refs = [color_attachment_ref_1, color_attachment_ref_2, color_attachment_ref_3, color_attachment_ref_4];

        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(4)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .build();
        
        let dependency_1 = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::MEMORY_READ)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dependency_flags(vk::DependencyFlags::BY_REGION)
            .build();

        let dependency_2 = vk::SubpassDependency::builder()
            .src_subpass(0)
            .dst_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .dependency_flags(vk::DependencyFlags::BY_REGION)
            .build();

        let dependencies = [dependency_1, dependency_2];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_descs)
            .subpasses(&[subpass])
            .dependencies(&dependencies)
            .build();

        let render_pass = unsafe {
            lve_device
                .device
                .create_render_pass(&render_pass_info, None).unwrap()
        };

        let image_views = [position.image_view, normal.image_view, albedo.image_view, metallic_roughness.image_view, depth.image_view];

        let frame_buffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&image_views)
            .width(width)
            .height(height)
            .layers(1);

        let framebuffer = unsafe {
            lve_device
                .device
                .create_framebuffer(&frame_buffer_info, None)
                .map_err(|e| log::error!("Unable to create render pass: {}", e))
                .unwrap()
        };

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .max_anisotropy(1.0)
            .min_lod(0.0)
            .max_lod(0.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .build();

        let sampler = unsafe {
            lve_device
                .device
                .create_sampler(&sampler_info, None)
                .map_err(|e| log::error!("Unable to create render pass: {}", e))
                .unwrap()
        };

        let things = Framebuffer {
            width,
            height,
            framebuffer,
            position,
            normal,
            albedo,
            metallic_roughness,
            depth,
            render_pass
        };

        DefferedRenderingSystem {
            lve_device,
            things,
            sampler
        }

    }

    pub fn get_render_pass(&self) -> vk::RenderPass {
        self.things.render_pass
    }

    fn create_attachment(lve_device: &Rc<LveDevice>, width: u32, height: u32, format: vk::Format, usage: vk::ImageUsageFlags) -> FrameBufferAttachment {
        let mut aspect_mask = vk::ImageAspectFlags::COLOR;
        let mut image_layout = vk::ImageLayout::UNDEFINED;

        if usage == vk::ImageUsageFlags::COLOR_ATTACHMENT {
            aspect_mask = vk::ImageAspectFlags::COLOR;
            image_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
        }

        if usage == vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT {
            aspect_mask = vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL;
            image_layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
        }

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {width, height, depth: 1})
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage | vk::ImageUsageFlags::SAMPLED);

        let (image, image_memory) = lve_device.create_image_with_info(&image_create_info, vk::MemoryPropertyFlags::DEVICE_LOCAL);

        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image,
        };

        let image_view = unsafe {
            lve_device.device
                .create_image_view(&imageview_create_info, None)
                .unwrap()
        };

        FrameBufferAttachment {
            image,
            image_memory,
            image_view,
            format
        }
    }

    pub fn start(&self, frame_info: &FrameInfo) {

        let color_clear_1 = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.01, 0.01, 0.01, 1.0],
            },
        };

        let color_clear_2 = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.01, 0.01, 0.01, 1.0],
            },
        };

        let color_clear_3 = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.01, 0.01, 0.01, 1.0],
            },
        };

        let color_clear_4 = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.01, 0.01, 0.01, 1.0],
            },
        };

        let depth_clear = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = [color_clear_1, color_clear_2, color_clear_3, color_clear_4, depth_clear];

        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: self.things.width, height: self.things.height}
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .clear_values(&clear_values)
            .render_pass(self.things.render_pass)
            .framebuffer(self.things.framebuffer)
            .render_area(render_area)
            .build();

        unsafe {
            self.lve_device.device.cmd_begin_render_pass(
                frame_info.command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            let viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(self.things.width as f32)
                .height(self.things.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build();

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width: self.things.width, height: self.things.height}
            };

            self.lve_device
                .device
                .cmd_set_viewport(frame_info.command_buffer, 0, &[viewport]);
            self.lve_device
                .device
                .cmd_set_scissor(frame_info.command_buffer, 0, &[scissor]);
        };
    }

    pub fn end(&self, frame_info: &FrameInfo) {
        unsafe {
            self.lve_device.device.cmd_end_render_pass(frame_info.command_buffer);
        }
    }
}

impl Drop for DefferedRenderingSystem {
    fn drop(&mut self) {
        log::debug!("Dropping SimpleRenderSystem");

        unsafe {
            self.lve_device.device.destroy_image(self.things.position.image, None);
            self.lve_device.device.destroy_image(self.things.normal.image, None);
            self.lve_device.device.destroy_image(self.things.albedo.image, None);
            self.lve_device.device.destroy_image(self.things.metallic_roughness.image, None);
            self.lve_device.device.destroy_image(self.things.depth.image, None);

            self.lve_device.device.free_memory(self.things.position.image_memory, None);
            self.lve_device.device.free_memory(self.things.normal.image_memory, None);
            self.lve_device.device.free_memory(self.things.albedo.image_memory, None);
            self.lve_device.device.free_memory(self.things.metallic_roughness.image_memory, None);
            self.lve_device.device.free_memory(self.things.depth.image_memory, None);

            self.lve_device.device.destroy_image_view(self.things.position.image_view, None);
            self.lve_device.device.destroy_image_view(self.things.normal.image_view, None);
            self.lve_device.device.destroy_image_view(self.things.albedo.image_view, None);
            self.lve_device.device.destroy_image_view(self.things.metallic_roughness.image_view, None);
            self.lve_device.device.destroy_image_view(self.things.depth.image_view, None);

            self.lve_device.device.destroy_sampler(self.sampler, None);
            self.lve_device.device.destroy_render_pass(self.things.render_pass, None);
            self.lve_device.device.destroy_framebuffer(self.things.framebuffer, None);
        }
    }
}
