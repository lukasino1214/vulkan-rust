use std::{collections::HashMap, rc::Rc};
use super::lve_device::*;

pub struct LveDescriptorSetLayout {
    lve_device: Rc<LveDevice>,
    pub layout: ash::vk::DescriptorSetLayout,
    pub bindings: HashMap<u32, ash::vk::DescriptorSetLayoutBinding>,
}

pub struct LveDescriptorSetLayoutBuilder {
    lve_device: Rc<LveDevice>,
    bindings: HashMap<u32, ash::vk::DescriptorSetLayoutBinding>,
}

impl LveDescriptorSetLayout {
    pub fn new(
        lve_device: Rc<LveDevice>,
    ) -> LveDescriptorSetLayoutBuilder {
        LveDescriptorSetLayoutBuilder {
            lve_device,
            bindings: HashMap::new(),
        }
    }
}

impl Drop for LveDescriptorSetLayout {
    fn drop(&mut self) {
        log::debug!("Dropping descriptor set layout");

        unsafe {
            self.lve_device.device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl LveDescriptorSetLayoutBuilder {
    pub fn add_binding(
        mut self,
        binding: u32,
        descriptor_type: ash::vk::DescriptorType,
        stage_flags: ash::vk::ShaderStageFlags,
        descriptor_count: u32,
    ) -> Self {
        assert_eq!(
            self.bindings.keys().filter(|&b| *b == binding).count(),
            0,
            "Binding already in use",
        );

        let layout_binding = ash::vk::DescriptorSetLayoutBinding {
            binding,
            descriptor_type,
            descriptor_count,
            stage_flags,
            ..Default::default()
        };

        self.bindings.insert(binding, layout_binding);

        self
    }

    pub fn build(self) -> Result<Rc<LveDescriptorSetLayout>, ash::vk::Result> {
        let LveDescriptorSetLayoutBuilder {
            lve_device,
            bindings
        } = self;

        let mut set_layout_bindings = Vec::new();
        for binding in bindings.values() {
            set_layout_bindings.push(*binding);
        }

        let layout_info = ash::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&set_layout_bindings);

        let layout = unsafe {
            lve_device.device.create_descriptor_set_layout(&layout_info, None)?
        };

        Ok(Rc::new(LveDescriptorSetLayout {
            lve_device,
            layout,
            bindings,
        }))
    }
}

pub struct LveDescriptorSetWriter {
    set_layout: Rc<LveDescriptorSetLayout>,
    pool: Rc<LveDescriptorPool>,
    writes: Vec<ash::vk::WriteDescriptorSet>,
}

impl LveDescriptorSetWriter {
    pub fn new(
        set_layout: Rc<LveDescriptorSetLayout>,
        pool: Rc<LveDescriptorPool>,
    ) -> Self {
        LveDescriptorSetWriter {
            set_layout,
            pool,
            writes: Vec::new(),
        }
    }

    pub fn write_to_buffer(
        mut self,
        binding: u32,
        buffer_info: &[ash::vk::DescriptorBufferInfo],
    ) -> Self {
        assert_eq!(
            self.set_layout.bindings.keys().filter(|&b| *b == binding).count(),
            1,
            "Layout does not contain specified binding",
        );

        let binding_description = self.set_layout.bindings[&binding];

        assert_eq!(
            binding_description.descriptor_count,
            1,
            "Binding single descriptor info, but binding expects multiple",
        );

        let write = ash::vk::WriteDescriptorSet::builder()
            .descriptor_type(binding_description.descriptor_type)
            .dst_binding(binding)
            .buffer_info(buffer_info)
            .build();

        self.writes.push(write);

        self
    }

    pub fn write_image(
        mut self,
        binding: u32,
        image_info: &[ash::vk::DescriptorImageInfo],
    ) -> Self {
        assert!(
            self.set_layout.bindings.keys().filter(|&b| *b == binding).count() == 1,
            "Layout does not contain specified binding",
        );

        let binding_description = self.set_layout.bindings[&binding];

        assert_eq!(
            binding_description.descriptor_count,
            1,
            "Binding single descriptor info, but binding expects multiple",
        );

        let write = ash::vk::WriteDescriptorSet::builder()
            .descriptor_type(binding_description.descriptor_type)
            .dst_binding(binding)
            .image_info(image_info)
            .build();

        self.writes.push(write);

        self
    }

    pub fn build(&mut self) -> Option<ash::vk::DescriptorSet> {
        let result = self.pool.allocate_descriptor(&[self.set_layout.layout]);
        //println!("error: {:?}", result);
        //result.map_err(|e| log::error!("Unable to read file: {}", e));

        if result.is_err() {
            //println!("error: {:?}", result);
            return None;
        }

        let set = self.overwrite(result.unwrap());
        Some(set)
    }

    pub fn overwrite(&mut self, set: ash::vk::DescriptorSet) -> ash::vk::DescriptorSet {
        for mut write in self.writes.iter_mut() {
            write.dst_set = set;
        }

        unsafe {
            self.pool.lve_device.device.update_descriptor_sets(&self.writes, &[]);
        }

        set
    }
}

pub struct LveDescriptorPool {
    pub lve_device: Rc<LveDevice>,
    pub pool: ash::vk::DescriptorPool,
}

pub struct LveDescriptorPoolBuilder {
    lve_device: Rc<LveDevice>,
    pool_sizes: Vec<ash::vk::DescriptorPoolSize>,
    max_sets: u32,
    pool_flags: ash::vk::DescriptorPoolCreateFlags,
}

impl LveDescriptorPool {
    pub fn new(
        lve_device: Rc<LveDevice>,
    ) -> LveDescriptorPoolBuilder {
        LveDescriptorPoolBuilder {
            lve_device,
            pool_sizes: Vec::new(),
            max_sets: 1000,
            pool_flags: ash::vk::DescriptorPoolCreateFlags::empty(),
        }
    }

    pub fn allocate_descriptor(
        &self,
        layouts: &[ash::vk::DescriptorSetLayout],
    ) -> Result<ash::vk::DescriptorSet, ash::vk::Result> {
        let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(layouts)
            .build();

        Ok(unsafe {
            self.lve_device.device.allocate_descriptor_sets(
                &alloc_info,
            )?[0]
        })
    }

    #[allow(dead_code)]
    pub fn free_descriptors(
        &self,
        descriptors: &Vec<ash::vk::DescriptorSet>
    ) -> Result<(), ash::vk::Result> {
        Ok(unsafe {
            self.lve_device.device.free_descriptor_sets(
                self.pool,
                descriptors,
            )?
        })
    }

    #[allow(dead_code)]
    pub fn reset_pool(&self) -> Result<(), ash::vk::Result> {
        Ok(unsafe {
            self.lve_device.device.reset_descriptor_pool(
                self.pool,
                ash::vk::DescriptorPoolResetFlags::empty(),
            )?
        })
    }
}

impl Drop for LveDescriptorPool {
    fn drop(&mut self) {
        log::debug!("Dropping descriptor pool");

        unsafe {
            self.lve_device.device.destroy_descriptor_pool(self.pool, None)
        }
    }
}

impl LveDescriptorPoolBuilder {
    pub fn add_pool_size(
        mut self,
        descriptor_type: ash::vk::DescriptorType,
        count: u32,
    ) -> Self {
        self.pool_sizes.push(
            ash::vk::DescriptorPoolSize {
                ty: descriptor_type,
                descriptor_count: count,
            }
        );

        self
    }

    #[allow(dead_code)]
    pub fn set_pool_flags(
        mut self,
        flags: ash::vk::DescriptorPoolCreateFlags,
    ) -> Self {
        self.pool_flags = flags;

        self
    }

    pub fn set_max_sets(
        mut self,
        max_sets: u32,
    ) -> Self {
        self.max_sets = max_sets;

        self
    }

    pub fn build(self) -> Result<Rc<LveDescriptorPool>, ash::vk::Result> {
        let LveDescriptorPoolBuilder {
            lve_device,
            pool_sizes,
            max_sets,
            pool_flags,
        } = self;
        
        let pool_info = ash::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(max_sets)
            .flags(pool_flags);

        let pool = unsafe {
            lve_device.device.create_descriptor_pool(&pool_info, None)?
        };

        Ok(Rc::new(LveDescriptorPool {
            lve_device,
            pool,
        }))
    }
}
