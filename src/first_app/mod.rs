mod lve_frame_info;
mod keyboard_movement_controller;
mod lve_camera;
mod lve_device;
mod lve_game_object;
mod lve_model;
mod lve_pipeline;
mod lve_renderer;
mod lve_swapchain;
mod lve_buffer;
mod simple_render_system;
mod lve_descriptor_set;

use keyboard_movement_controller::*;
use lve_camera::*;
use lve_device::*;
use lve_game_object::*;
use lve_model::*;
use lve_renderer::*;
use simple_render_system::*;
use lve_frame_info::*;
use lve_descriptor_set::*;

use winit::{
    dpi::{LogicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use lve_buffer::*;
use lve_swapchain::*;

use winit::event::VirtualKeyCode;

use std::{rc::Rc};

extern crate nalgebra as na;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const NAME: &str = "Vulkan but with Rust 👀";

/*#[derive(PartialEq)]
struct GlobalUbo {
    projection_matrix: na::Matrix4<f32>,
    light_direction: na::Vector3<f32>
}*/

pub struct VulkanApp {
    pub window: Window,
    lve_renderer: LveRenderer,
    simple_render_system: SimpleRenderSystem,
    game_objects: Vec<LveGameObject>,
    viewer_object: LveGameObject,
    camera_controller: KeyboardMovementController,
    global_pool: Rc<LveDescriptorPool>,
    global_set_layout: Rc<LveDescriptorSetLayout>,
    global_descriptor_sets: Vec<ash::vk::DescriptorSet>,
    ubo_buffers: Vec<LveBuffer<GlobalUbo>>
}

impl VulkanApp {
    pub fn new() -> (Self, EventLoop<()>) {
        let (event_loop, window) = Self::new_window(WIDTH, HEIGHT, NAME);

        let lve_device = LveDevice::new(&window);

        let lve_renderer = LveRenderer::new(Rc::clone(&lve_device), &window);

        let game_objects = Self::load_game_objects(&lve_device);

        let viewer_object = LveGameObject::new(
            LveModel::new_null(Rc::clone(&lve_device), "camera"),
            None,
            None,
        );

        let camera_controller = KeyboardMovementController::new(Some(100.0), Some(100.0));

        let global_pool = LveDescriptorPool::new(Rc::clone(&lve_device))
            .set_max_sets(MAX_FRAMES_IN_FLIGHT as u32)
            .add_pool_size(ash::vk::DescriptorType::UNIFORM_BUFFER, MAX_FRAMES_IN_FLIGHT as u32)
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

        let simple_render_system = SimpleRenderSystem::new(
            Rc::clone(&lve_device),
            &lve_renderer.get_swapchain_render_pass(),
            &[global_set_layout.layout]
        );

        (
            Self {
                window,
                lve_renderer,
                simple_render_system,
                game_objects,
                viewer_object,
                camera_controller,
                global_pool,
                global_set_layout,
                global_descriptor_sets,
                ubo_buffers
            },
            event_loop,
        )
    }

    pub fn run(
        &mut self,
        keys_pressed: &[VirtualKeyCode],
        frame_time: f32,
    ) {
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
            .set_perspective_projection(50_f32.to_radians(), aspect, 0.1, 10.0)
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

        match self.lve_renderer.begin_frame(&self.window) {
            Some(command_buffer) => {
                let frame_index = self.lve_renderer.get_frame_index();
                let frame_info = FrameInfo {
                    frame_index,
                    frame_time,
                    command_buffer,
                    camera,
                    global_descriptor_set: self.global_descriptor_sets[frame_index]
                };

                let ubo = GlobalUbo {
                    projection_matrix: frame_info.camera.projection_matrix,
                    view_matrix: frame_info.camera.view_matrix,
                    light_direction: na::vector!(1.0, -3.0, -1.0).normalize()
                };

                self.ubo_buffers[frame_index].write_to_buffer(&[ubo]);
                //self.ubo_buffers[frame_index].flush();

                self.lve_renderer
                    .begin_swapchain_render_pass(command_buffer);
                self.simple_render_system.render_game_objects(
                    &frame_info,
                    &mut self.game_objects,
        
                );
                self.lve_renderer.end_swapchain_render_pass(command_buffer);
            }
            None => {}
        }

        self.lve_renderer.end_frame();
    }

    pub fn resize(&mut self) {
        self.lve_renderer.recreate_swapchain(&self.window)
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
        let lve_model = LveModel::new_from_file(Rc::clone(lve_device), "cube", "./models/smooth_vase.obj");

        let transform = Some(TransformComponent {
            translation: na::vector![0.0, 0.0, 2.5],
            scale: na::vector![0.5, 0.5, 0.5],
            rotation: na::vector![0.0, 0.0, 0.0],
        });

        vec![LveGameObject::new(lve_model, None, transform)]
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        log::debug!("Dropping application");
    }
}
