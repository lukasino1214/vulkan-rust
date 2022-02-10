use super::lve_device::LveDevice;
use super::lve_model::*;

use ash::{vk, Device};

use std::ffi::CString;
use std::rc::Rc;

pub struct PipelineConfigInfo {
    viewport_info: vk::PipelineViewportStateCreateInfo,
    input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo,
    rasterization_info: vk::PipelineRasterizationStateCreateInfo,
    multisample_info: vk::PipelineMultisampleStateCreateInfo,
    _color_blend_attachment: Rc<vk::PipelineColorBlendAttachmentState>,
    color_blend_info: Rc<vk::PipelineColorBlendStateCreateInfo>,
    depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo,
    _dynamic_state_enables: Vec<vk::DynamicState>,
    dynamic_state_info: vk::PipelineDynamicStateCreateInfo,
    subpass: u32,
}

pub struct LvePipeline {
    lve_device: Rc<LveDevice>,
    graphics_pipeline: vk::Pipeline,
    vert_shader_module: vk::ShaderModule,
    frag_shader_module: vk::ShaderModule,
}

impl LvePipeline {
    pub fn new(
        lve_device: Rc<LveDevice>,
        vert_file_path: &str,
        frag_file_path: &str,
        config_info: PipelineConfigInfo,
        render_pass: &vk::RenderPass,
        pipeline_layout: &vk::PipelineLayout,
    ) -> Self {
        let (graphics_pipeline, vert_shader_module, frag_shader_module) =
            Self::create_graphics_pipeline(
                &lve_device.device,
                vert_file_path,
                frag_file_path,
                config_info,
                render_pass,
                pipeline_layout,
            );

        Self {
            lve_device,
            graphics_pipeline,
            vert_shader_module,
            frag_shader_module,
        }
    }

    pub unsafe fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline,
        );
    }

    pub fn default_pipline_config_info() -> PipelineConfigInfo {
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST) 
            .primitive_restart_enable(false) 
            .build();

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false) 
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE) 
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0) // optional
            .depth_bias_clamp(0.0) // optional
            .depth_bias_slope_factor(0.0) // optional
            .build();

        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0) // optional
            // .sample_mask()                       // optional
            .alpha_to_coverage_enable(false) // optional
            .alpha_to_one_enable(false) // optional
            .build();

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .build();

        let color_blend_attachment = Rc::new(vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE) // optional
            .dst_color_blend_factor(vk::BlendFactor::ZERO) // optional
            .color_blend_op(vk::BlendOp::ADD) // optional
            .src_alpha_blend_factor(vk::BlendFactor::ONE) // optional
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO) // optional
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()); // optional

        let color_blend_info = Rc::new(vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY) // optional
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build()); // optional

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0) // optional
            .max_depth_bounds(1.0) // optional
            .stencil_test_enable(false)
            // .front()                                 // optional
            // .back()                                  // optional
            .build();

        let dynamic_state_enables = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_state_enables)
            .flags(vk::PipelineDynamicStateCreateFlags::empty())
            .build();

        PipelineConfigInfo {
            viewport_info,
            input_assembly_info,
            rasterization_info,
            multisample_info,
            _color_blend_attachment: color_blend_attachment,
            color_blend_info,
            depth_stencil_info,
            _dynamic_state_enables: dynamic_state_enables,
            dynamic_state_info,
            subpass: 0,
        }
    }

    fn create_graphics_pipeline(
        device: &Device,
        vert_file_path: &str,
        frag_file_path: &str,
        config_info: PipelineConfigInfo,
        render_pass: &vk::RenderPass,
        pipeline_layout: &vk::PipelineLayout,
    ) -> (vk::Pipeline, vk::ShaderModule, vk::ShaderModule) {
        assert_ne!(
            pipeline_layout,
            &vk::PipelineLayout::null(),
            "Cannot create graphics pipeline:: no pipeline_layout provided in config_info"
        );
        assert_ne!(
            render_pass,
            &vk::RenderPass::null(),
            "Cannot create graphics pipeline:: no render_pass provided in config_info"
        );

        let vert_source: &str = &std::fs::read_to_string(vert_file_path).expect("Something went wrong reading the file");
        let frag_source: &str = &std::fs::read_to_string(frag_file_path).expect("Something went wrong reading the file");

        let mut compiler = shaderc::Compiler::new().unwrap();

        let vert_code = compiler.compile_into_spirv(vert_source, shaderc::ShaderKind::Vertex,vert_file_path, "main", None).unwrap().as_binary().to_vec();
        let frag_code = compiler.compile_into_spirv(frag_source, shaderc::ShaderKind::Fragment,frag_file_path, "main", None).unwrap().as_binary().to_vec();

        let vert_shader_module = Self::create_shader_module(device, &vert_code);
        let frag_shader_module = Self::create_shader_module(device, &frag_code);

        let entry_point_name = CString::new("main").unwrap();

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&entry_point_name)
            // .flags(vk::PipelineShaderStageCreateFlags::empty())
            // .next()
            // .specialization_info()
            .build();

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(&entry_point_name)
            // .flags(vk::PipelineShaderStageCreateFlags::empty())
            // .next()
            // .specialization_info()
            .build();

        let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

        let binding_descriptions = Vertex::get_binding_descriptions();
        let attribute_descriptions = Vertex::get_attribute_descriptions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&config_info.input_assembly_info)
            .viewport_state(&config_info.viewport_info)
            .rasterization_state(&config_info.rasterization_info)
            .multisample_state(&config_info.multisample_info)
            .color_blend_state(&config_info.color_blend_info)
            .depth_stencil_state(&config_info.depth_stencil_info)
            .dynamic_state(&config_info.dynamic_state_info)
            .layout(*pipeline_layout)
            .render_pass(*render_pass)
            .subpass(config_info.subpass)
            .base_pipeline_index(-1)
            .base_pipeline_handle(vk::Pipeline::null());

        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_info), None)
                .map_err(|e| log::error!("Unable to create graphics pipeline: {:?}", e))
                .unwrap()[0]
        };

        (graphics_pipeline, vert_shader_module, frag_shader_module)
    }

    fn create_shader_module(device: &Device, code: &Vec<u32>) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code).build();

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .map_err(|e| log::error!("Unable to create shader module: {}", e))
                .unwrap()
        }
    }
}

impl Drop for LvePipeline {
    fn drop(&mut self) {
        log::debug!("Dropping pipeline");

        unsafe {
            self.lve_device.device.destroy_shader_module(self.vert_shader_module, None);
            self.lve_device.device.destroy_shader_module(self.frag_shader_module, None);
            self.lve_device.device.destroy_pipeline(self.graphics_pipeline, None);
        }
    }
}
