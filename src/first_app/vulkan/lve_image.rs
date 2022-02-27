use super::lve_device::*;
use super::lve_buffer::*;

use ash::vk;

use std::rc::Rc;

pub struct LveImage {
    lve_device: Rc<LveDevice>,
    image: vk::Image,
    image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub image_sampler: vk::Sampler,
    pub image_info: vk::DescriptorImageInfo
}

impl LveImage {
    pub fn default(lve_device: Rc<LveDevice>) -> Self {
        Self::new(lve_device, "./assets/textures/default_texture.png")
    }

    #[allow(dead_code)]
    pub fn null(lve_device: Rc<LveDevice>) -> Self {

        let image_view = ash::vk::ImageView::null();
        let image_sampler = ash::vk::Sampler::null();

        let image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(image_sampler)
            .build();

        LveImage {
            lve_device,
            image: ash::vk::Image::null(),
            image_memory: ash::vk::DeviceMemory::null(),
            image_view,
            image_sampler,
            image_info
        }
    }

    pub fn new(lve_device: Rc<LveDevice>, path: &str) -> Self {
        let image = image::open(path).map(|img| img.to_rgba8()).expect("unable to open image");
        let (width, height) = image.dimensions();
        let size = (width * height * 4) as usize;

        let mut staging_buffer = LveBuffer::new(lve_device.clone() , size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        staging_buffer.map(0);
        staging_buffer.write_to_buffer(&image.into_raw());
        staging_buffer.unmap();

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {width, height, depth: 1})
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED);

        let (image, image_memory) = lve_device.create_image_with_info(&image_create_info, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        Self::transition_image_layout(&lve_device, image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL, 1);
        lve_device.copy_buffer_to_image(staging_buffer.buffer, image, width, height, 1);
        Self::transition_image_layout(&lve_device, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 1);

        let image_view = Self::create_image_view(&lve_device, image, vk::Format::R8G8B8A8_SRGB);
        let image_sampler = Self::create_texture_sampler(&lve_device);

        let image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(image_sampler)
            .build();

        LveImage {
            lve_device,
            image,
            image_memory,
            image_view,
            image_sampler,
            image_info
        }
    }

    fn transition_image_layout(lve_device: &Rc<LveDevice>, image: vk::Image, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, mip_levels: u32) {
        let command_buffer = lve_device.begin_single_time_commands();

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask =
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: std::ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        unsafe {
            lve_device.device.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }

        lve_device.end_single_time_commands(command_buffer);
    }

    fn create_image_view(
        lve_device: &Rc<LveDevice>,
        image: vk::Image,
        format: vk::Format,
    ) -> vk::ImageView {
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
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image,
        };

        unsafe {
            lve_device.device
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        }
    }

    fn create_texture_sampler(lve_device: &Rc<LveDevice>) -> vk::Sampler {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: 0.0,
            anisotropy_enable: vk::FALSE,
            max_anisotropy: 16.0,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };

        unsafe {
            lve_device.device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }
}

impl Drop for LveImage {
    fn drop(&mut self) {
        log::debug!("Dropping descriptor pool");

        unsafe {
            self.lve_device.device.destroy_image(self.image, None);
            self.lve_device.device.free_memory(self.image_memory, None);
            self.lve_device.device.destroy_image_view(self.image_view, None);
            self.lve_device.device.destroy_sampler(self.image_sampler, None);
        }
    }
}