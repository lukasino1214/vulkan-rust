mod ecs;
mod systems;
mod vulkan;
mod keyboard_movement_controller;

use systems::{advanced_render_system::*, simple_render_system::*, point_render_system::*, deffered_rendering_system::*, composition_render_system::*};
use vulkan::{lve_camera::*, lve_device::*, lve_game_object::*, lve_model::*, lve_renderer::*, lve_frame_info::*, lve_descriptor_set::*, lve_image::*, lve_buffer::*, lve_swapchain::*};
use ecs::{scene::*, entity::*, model::*};
use keyboard_movement_controller::*;

use winit::{
    dpi::{LogicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};


use winit::event::VirtualKeyCode;

use imgui::*;
use imgui_winit_support::*;
use imgui_rs_vulkan_renderer::*;

use std::{rc::Rc};

extern crate nalgebra as na;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const NAME: &str = "Vulkan but with Rust 👀";

#[allow(dead_code)]
pub struct VulkanApp {
    pub window: Window,
    lve_renderer: LveRenderer,
    simple_render_system: SimpleRenderSystem,
    advanced_render_system: AdvancedRenderSystem,
    point_render_system: PointRenderSystem,
    game_objects: Vec<LveGameObject>,
    viewer_object: LveGameObject,
    camera_controller: KeyboardMovementController,
    global_pool: Rc<LveDescriptorPool>,
    global_set_layout: Rc<LveDescriptorSetLayout>,
    global_descriptor_sets: Vec<ash::vk::DescriptorSet>,
    ubo_buffers: Vec<LveBuffer<GlobalUbo>>,
    image_set_layout: Rc<LveDescriptorSetLayout>,
    image: LveImage,
    image_descriptor_set: ash::vk::DescriptorSet,
    pub imgui: Context,
    font_size: f32,
    pub platform: WinitPlatform,
    renderer: Renderer,
    lve_device: Rc<LveDevice>,
    rebuild: bool,
    scene: Scene,
    deffered_rendering_system: DefferedRenderingSystem,
    composition_render_system: CompositionRenderSystem,
    deffered_set_layout: Rc<LveDescriptorSetLayout>,
    deffered_descriptor_set: ash::vk::DescriptorSet
}

impl VulkanApp {
    pub fn new() -> (Self, EventLoop<()>) {
        let (event_loop, window) = Self::new_window(WIDTH, HEIGHT, NAME);

        let lve_device = LveDevice::new(&window);

        let lve_renderer = LveRenderer::new(Rc::clone(&lve_device), &window);

        let game_objects = Self::load_game_objects(&lve_device);

        let camera_transform = Some(TransformComponent {
            translation: na::vector![0.0, -0.20, -1.0],
            scale: na::vector![0.5, 0.5, 0.5],
            rotation: na::vector![0.0, 0.0, 0.0],
        });

        let viewer_object = LveGameObject::new(
            None,
            None,
            camera_transform,
        );

        let camera_controller = KeyboardMovementController::new(Some(500.0), Some(500.0));

        let global_pool = LveDescriptorPool::new(Rc::clone(&lve_device))
            .set_max_sets(1000 as u32)
            .add_pool_size(ash::vk::DescriptorType::UNIFORM_BUFFER, MAX_FRAMES_IN_FLIGHT as u32)
            .add_pool_size(ash::vk::DescriptorType::SAMPLED_IMAGE, MAX_FRAMES_IN_FLIGHT as u32)
            .add_pool_size(ash::vk::DescriptorType::STORAGE_BUFFER, MAX_FRAMES_IN_FLIGHT as u32)
            .build().unwrap();

        let mut ubo_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let mut buffer = LveBuffer::new(
                Rc::clone(&lve_device),
                1,
                ash::vk::BufferUsageFlags::UNIFORM_BUFFER,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE,
            );

            buffer.map(0);

            ubo_buffers.push(buffer);
        }

        let global_set_layout = LveDescriptorSetLayout::new(Rc::clone(&lve_device))
            .add_binding(0, ash::vk::DescriptorType::UNIFORM_BUFFER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .build().unwrap();

        let mut global_descriptor_sets = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = ubo_buffers[i].descriptor_info();
            let set = LveDescriptorSetWriter::new(global_set_layout.clone(), global_pool.clone())
                .write_to_buffer(0, &[buffer_info])
                .build().unwrap();

            global_descriptor_sets.push(set);
        }

        let image_set_layout = LveDescriptorSetLayout::new(Rc::clone(&lve_device))
            .add_binding(0, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .build().unwrap();

        let descriptor_layout = LveDescriptorSetLayout::new(Rc::clone(&lve_device))
            .add_binding(0, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(1, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(2, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(3, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(4, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(5, ash::vk::DescriptorType::UNIFORM_BUFFER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .build().unwrap();

        let image = LveImage::new(Rc::clone(&lve_device), "./assets/textures/poggers.png");

        let image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image.image_view)
            .sampler(image.image_sampler)
            .build();

        let image_descriptor_set = LveDescriptorSetWriter::new(image_set_layout.clone(), global_pool.clone())
                .write_image(0, &[image_info])
                .build().unwrap();

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (hidpi_factor) as f32;
        let monitor_size = window.current_monitor().unwrap().size();
        imgui.io_mut().display_size = [monitor_size.width as f32, monitor_size.height as f32];
        imgui.io_mut().display_size = [1920.0, 1080.0];
        imgui.io_mut().font_global_scale = (1.00 / hidpi_factor) as f32;
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

        let renderer = Renderer::with_default_allocator(
            &lve_device.instance,
            lve_device.physical_device,
            lve_device.device.clone(),
            lve_device.graphics_queue,
            lve_device.command_pool,
            lve_renderer.get_swapchain_render_pass(),
            &mut imgui,
            Some(Options {
                in_flight_frames: 2,
                ..Default::default()
            }),
        ).unwrap();

        let mut scene = Scene::new_null("test");
        let mut entity_1 = Entity::new("test_1", NewTransformComponent { translation: na::vector![0.0, 0.0, 3.0], rotation: na::vector![0.0, 0.0, 3.141], scale: na::vector![0.01, 0.01, 0.01]});
        let model = Rc::new(Model::new(&Rc::clone(&lve_device), "./assets/models/Sponza/glTF/Sponza.gltf", global_pool.clone()));
        //let model = Rc::new(Model::new(&Rc::clone(&lve_device), "./assets/models/Map/scene.gltf", global_pool.clone()));
        //let model = Rc::new(Model::new(&Rc::clone(&lve_device), "./assets/models/deccer-cubes/SM_Deccer_Cubes_Textured.gltf", global_pool.clone()));
        entity_1.set_model(model);
        let mut entity_2 = Entity::new("test_2", NewTransformComponent { translation: na::vector![0.0, 0.0, 0.0], rotation: na::vector![0.0, 0.0, 0.0], scale: na::vector![0.0, 0.0, 0.0]});
        entity_2.add_spot_light();
        let mut entity_3 = Entity::new("test_3", NewTransformComponent { translation: na::vector![0.0, 0.0, 0.0], rotation: na::vector![0.0, 0.0, 0.0], scale: na::vector![0.0, 0.0, 0.0]});
        entity_3.add_directional_light();

        scene.add_entity(entity_1);
        scene.add_entity(entity_2);

        println!("deffered");
        let deffered_rendering_system = DefferedRenderingSystem::new(lve_device.clone(), WIDTH, HEIGHT);

        let deffered_set_layout = LveDescriptorSetLayout::new(Rc::clone(&lve_device))
            .add_binding(0, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(1, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(2, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(3, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .build().unwrap();

        let position_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(deffered_rendering_system.things.position.image_view)
            .sampler(deffered_rendering_system.sampler)
            .build();

        let normal_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(deffered_rendering_system.things.normal.image_view)
            .sampler(deffered_rendering_system.sampler)
            .build();

        let albedo_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(deffered_rendering_system.things.albedo.image_view)
            .sampler(deffered_rendering_system.sampler)
            .build();

        let metallic_roughness_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(deffered_rendering_system.things.metallic_roughness.image_view)
            .sampler(deffered_rendering_system.sampler)
            .build();

        let deffered_descriptor_set = LveDescriptorSetWriter::new(deffered_set_layout.clone(), global_pool.clone())
                .write_image(0, &[position_image_info])
                .write_image(1, &[normal_image_info])
                .write_image(2, &[albedo_image_info])
                .write_image(3, &[metallic_roughness_image_info])
                .build().unwrap();

        println!("simple");
        let simple_render_system = SimpleRenderSystem::new(
            Rc::clone(&lve_device),
            &lve_renderer.get_swapchain_render_pass(),
            &[global_set_layout.layout, descriptor_layout.layout]
        );

        println!("advanced");
        let advanced_render_system = AdvancedRenderSystem::new(
            Rc::clone(&lve_device),
            &deffered_rendering_system.get_render_pass(),
            &[global_set_layout.layout, descriptor_layout.layout]
        );

        println!("point");
        let point_render_system = PointRenderSystem::new(
            Rc::clone(&lve_device),
            &deffered_rendering_system.get_render_pass(),
            &[global_set_layout.layout]
        );

        println!("composition");
        let composition_render_system = CompositionRenderSystem::new(
            Rc::clone(&lve_device),
            &lve_renderer.get_swapchain_render_pass(),
            &[global_set_layout.layout, deffered_set_layout.layout]
        );

        (
            Self {
                window,
                lve_renderer,
                simple_render_system,
                advanced_render_system,
                point_render_system,
                game_objects,
                viewer_object,
                camera_controller,
                global_pool,
                global_set_layout,
                global_descriptor_sets,
                ubo_buffers,
                image_set_layout,
                image,
                image_descriptor_set,
                imgui,
                font_size,
                platform,
                renderer,
                lve_device,
                rebuild: false,
                scene,
                deffered_rendering_system,
                composition_render_system,
                deffered_set_layout,
                deffered_descriptor_set
            },
            event_loop,
        )
    }

    pub fn run(&mut self, keys_pressed: &[VirtualKeyCode], frame_time: f32) {
        // log::debug!("frame time: {}s", frame_time);
        // log::debug!("Keys pressed: {:?}", keys_pressed);
        // log::debug!("fps: {:?}", 1.0/frame_time); // This is a bit shit :)
        // println!("fps: {:?}", 1.0/frame_time); // This is a bit shit :)
        // println!("frame time: {:?}", frame_time);

        self.camera_controller.move_in_plane_xz(
            keys_pressed,
            frame_time,
            &mut self.viewer_object,
        );

        let aspect = self.lve_renderer.get_aspect_ratio();
        // self.camera = LveCamera::set_orthographic_projection(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        let camera = LveCameraBuilder::new()
            .set_view_xyz(
                self.viewer_object.transform.translation,
                self.viewer_object.transform.rotation,
            )
            .set_perspective_projection(70_f32.to_radians(), aspect, 0.001, 10000000.0)
            // .set_view_direction(na::Vector3::zeros(), na::vector![0.5, 0.0, 1.0], None)
            // .set_view_target(
            //     na::vector![-1.0, -2.0, 2.0],
            //     na::vector![0.0, 0.0, 2.5],
            //     None,
            // )
            .build();

        let extent = LveRenderer::get_window_extent(&self.window);

        if extent.width == 0 || extent.height == 0 {
            return;
        }

        if self.rebuild {
            self.simple_render_system.recreate_pipeline(self.lve_device.clone(), &self.lve_renderer.get_swapchain_render_pass());
            self.point_render_system.recreate_pipeline(self.lve_device.clone(), &self.deffered_rendering_system.get_render_pass());
            self.advanced_render_system.recreate_pipeline(self.lve_device.clone(), &self.deffered_rendering_system.get_render_pass());
            self.composition_render_system.recreate_pipeline(self.lve_device.clone(), &self.lve_renderer.get_swapchain_render_pass());
            self.rebuild = false;
        }

        match self.lve_renderer.begin_frame(&self.window) {
            Some(command_buffer) => {
                let frame_index = self.lve_renderer.get_frame_index();
                let frame_info = FrameInfo {
                    frame_index,
                    frame_time,
                    command_buffer,
                    camera,
                    global_descriptor_set: self.global_descriptor_sets[frame_index],
                    image_descriptor_set: self.image_descriptor_set,
                    game_objects: &self.game_objects
                };

                let mut ubo = GlobalUbo {
                    projection_matrix: Align16(frame_info.camera.projection_matrix),
                    view_matrix: Align16(frame_info.camera.view_matrix),
                    camera_position: Align16(self.viewer_object.transform.translation),
                    ambient_light_color: Align16(na::vector![1.0, 1.0, 1.0, 0.02]),
                    point_lights: [PointLight { position: na::vector![0.0,0.0,0.0,0.0], color: na::vector![0.0,0.0,0.0,0.0] }; MAX_LIGHTS],
                    num_lights: 0
                };

                self.point_render_system.update(&frame_info, &mut ubo);

                self.ubo_buffers[frame_index].write_to_buffer(&[ubo]);
                //self.ubo_buffers[frame_index].flush();

                self.deffered_rendering_system.start(&frame_info);
                self.advanced_render_system.render_scene(&frame_info, &self.scene);
                self.point_render_system.render(&frame_info);
                self.deffered_rendering_system.end(&frame_info);

                self.lve_renderer.begin_swapchain_render_pass(command_buffer);
                //self.simple_render_system.render_game_objects(&frame_info);
                //self.advanced_render_system.render_scene(&frame_info, &self.scene);
                //self.point_render_system.render(&frame_info);

                self.composition_render_system.render(&frame_info, self.deffered_descriptor_set);



                self.platform.prepare_frame(self.imgui.io_mut(), &self.window).expect("Failed to prepare frame");
                let ui = self.imgui.frame();

                imgui::Window::new("utils")
                    .size([300.0, 100.0], Condition::FirstUseEver)
                    .build(&ui, || {
                        if ui.button("rebuild pipelines") {
                            println!("rebuilding pipelines");
                            self.rebuild = true;
                        }
                        /*ui.separator();
                        let mouse_pos = ui.io().mouse_pos;
                        ui.text(format!(
                            "Mouse Position: ({:.1},{:.1})",
                            mouse_pos[0], mouse_pos[1]
                        ));*/
                    });

                self.scene.display_info(&ui);

                self.platform.prepare_render(&ui, &self.window);
                let draw_data = ui.render();
                self.renderer.cmd_draw(command_buffer, draw_data).unwrap();

                self.lve_renderer.end_swapchain_render_pass(command_buffer);
            }
            None => {}
        }

        self.lve_renderer.end_frame(&self.window);
    }

    pub fn resize(&mut self) {
        self.lve_renderer.recreate_swapchain(&self.window);

        let window_inner_size = self.window.inner_size();
        println!("{:?}", window_inner_size);
        self.deffered_rendering_system = DefferedRenderingSystem::new(Rc::clone(&self.lve_device), window_inner_size.width, window_inner_size.height);

        let deffered_set_layout = LveDescriptorSetLayout::new(Rc::clone(&self.lve_device))
            .add_binding(0, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(1, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(2, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .add_binding(3, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, ash::vk::ShaderStageFlags::ALL_GRAPHICS, 1)
            .build().unwrap();

        let position_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.deffered_rendering_system.things.position.image_view)
            .sampler(self.deffered_rendering_system.sampler)
            .build();

        let normal_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.deffered_rendering_system.things.normal.image_view)
            .sampler(self.deffered_rendering_system.sampler)
            .build();

        let albedo_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.deffered_rendering_system.things.albedo.image_view)
            .sampler(self.deffered_rendering_system.sampler)
            .build();

        let metallic_roughness_image_info = ash::vk::DescriptorImageInfo::builder()
            .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.deffered_rendering_system.things.metallic_roughness.image_view)
            .sampler(self.deffered_rendering_system.sampler)
            .build();

        let deffered_descriptor_set = LveDescriptorSetWriter::new(deffered_set_layout.clone(), self.global_pool.clone())
            .write_image(0, &[position_image_info])
            .write_image(1, &[normal_image_info])
            .write_image(2, &[albedo_image_info])
            .write_image(3, &[metallic_roughness_image_info])
            .build().unwrap();

        self.deffered_descriptor_set = deffered_descriptor_set;
        self.deffered_set_layout = deffered_set_layout;

        self.composition_render_system = CompositionRenderSystem::new(
            Rc::clone(&self.lve_device),
            &self.lve_renderer.get_swapchain_render_pass(),
            &[self.global_set_layout.layout, self.deffered_set_layout.layout]
        );
    }

    fn new_window(w: u32, h: u32, name: &str) -> (EventLoop<()>, Window) {
        log::debug!("Starting event loop");
        let event_loop = EventLoop::new();

        log::debug!("Creating window");
        let winit_window = WindowBuilder::new()
            .with_title(name)
            .with_inner_size(LogicalSize::new(w, h))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();

        (event_loop, winit_window)
    }

    fn load_game_objects(lve_device: &Rc<LveDevice>) -> Vec<LveGameObject> {
        let vase = LveModel::new_from_file(Rc::clone(lve_device), "./assets/models/smooth_vase.obj");

        let vase_transform = Some(TransformComponent {
            translation: na::vector![0.0, 0.0, 0.2],
            scale: na::vector![0.5, 0.5, 0.5],
            rotation: na::vector![0.0, 0.0, 0.0],
        });

        /*let floor = LveModel::new_from_file(Rc::clone(lve_device), "./assets/models/sponza.obj");

        let floor_transform = Some(TransformComponent {
            translation: na::vector![0.0, 0.5, 0.0],
            scale: na::vector![1.0, 1.0, 1.0],
            rotation: na::vector![0.0, 0.0, 3.141],
        });*/

        let mut game_objects: Vec<LveGameObject> = Vec::new();

        game_objects.push(LveGameObject::new(Some(vase), None, vase_transform));
        //game_objects.push(LveGameObject::new(Some(floor), None, floor_transform));

        let light_colors = vec![
            na::vector![1.0, 0.1, 0.1],
            na::vector![0.1, 0.1, 1.0],
            na::vector![0.1, 1.0, 0.1],
            na::vector![1.0, 1.0, 0.1],
            na::vector![0.1, 1.0, 1.0],
            na::vector![1.0, 1.0, 1.0],
        ];

        game_objects.push(LveGameObject::make_point_light(0.5, 0.05, light_colors[0]));
        game_objects[1].transform.translation = na::vector![0.0, -0.4, 2.0];
        game_objects.push(LveGameObject::make_point_light(0.5, 0.05, light_colors[1]));
        game_objects[2].transform.translation = na::vector![4.5, -0.4, 2.0];
        game_objects.push(LveGameObject::make_point_light(0.5, 0.05, light_colors[2]));
        game_objects[3].transform.translation = na::vector![-4.5, -0.4, 2.0];

        game_objects
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        log::debug!("Dropping application");
    }
}
