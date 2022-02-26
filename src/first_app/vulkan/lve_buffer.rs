use std::{marker::PhantomData, ffi::c_void, rc::Rc};
use super::lve_device::*;
pub struct LveBuffer<T>
where
    T: PartialEq,
{
    lve_device: Rc<LveDevice>,
    pub mapped: Option<*mut c_void>,
    pub buffer: ash::vk::Buffer,
    memory: ash::vk::DeviceMemory,
    capacity: usize,

    _p: PhantomData<T>,
}

impl<T> LveBuffer<T>
where
    T: PartialEq,
{
    pub fn new(
        lve_device: Rc<LveDevice>,
        size: usize,
        usage_flags: ash::vk::BufferUsageFlags,
        memory_property_flags: ash::vk::MemoryPropertyFlags,
    ) -> Self {
        let byte_len = std::mem::size_of::<T>() * size;

        let (buffer, memory) = lve_device.create_buffer(
            byte_len as u64,
            usage_flags,
            memory_property_flags
        );

        Self {
            lve_device,
            mapped: None,
            buffer,
            memory,
            capacity: size,

            _p: PhantomData {},
        }
    }

    pub fn null(lve_device: Rc<LveDevice>) -> Self {
        let size = 0;

        let buffer = ash::vk::Buffer::null();
        let memory = ash::vk::DeviceMemory::null();

        Self {
            lve_device,
            mapped: None,
            buffer,
            memory,
            capacity: size,

            _p: PhantomData {},
        }
    }

    pub fn bind_vertex(&self, command_buffer: ash::vk::CommandBuffer) {
        unsafe {
            self.lve_device.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.buffer], &[0])
        }
    }

    pub fn bind_index(&self, command_buffer: ash::vk::CommandBuffer, index_type: ash::vk::IndexType) {
        unsafe {
            self.lve_device.device.cmd_bind_index_buffer(command_buffer, self.buffer, 0, index_type)
        }
    }

    pub fn map(&mut self, element_offset: usize) /*-> Result<(), ash::vk::Result>*/ {
        let size = self.capacity - element_offset;
        let mem_size = (std::mem::size_of::<T>() * size) as u64;
        let mem_offset = (std::mem::size_of::<T>() * element_offset) as u64;

        unsafe {
            self.mapped = Some(self.lve_device.device.map_memory(
                self.memory,
                mem_offset,
                mem_size,
                ash::vk::MemoryMapFlags::empty()
            ).unwrap() )
        };
    }

    pub fn unmap(&mut self) {
        match self.mapped {
            Some(_) => {
                unsafe {
                    self.lve_device.device.unmap_memory(self.memory);
                }

                self.mapped = None;
            },
            None => { },
        }
    }

    pub fn write_to_buffer(&mut self, elements: &[T]) {
        unsafe {
            elements
                .as_ptr()
                .copy_to_nonoverlapping(self.mapped.unwrap() as *mut _, elements.len());
        }
    }

    #[allow(dead_code)]
    pub fn flush(&self) {
        let mapped_range = [ash::vk::MappedMemoryRange::builder()
            .memory(self.memory)
            .offset(0)
            .size(ash::vk::WHOLE_SIZE)
            .build()];

        unsafe {
            self.lve_device.device.flush_mapped_memory_ranges(&mapped_range)
            .map_err(|e| log::error!("Unable to flush buffer: {}", e))
                .unwrap();
        }
    }

    pub fn descriptor_info(&self) -> ash::vk::DescriptorBufferInfo {
        ash::vk::DescriptorBufferInfo::builder()
            .buffer(self.buffer)
            .offset(0)
            .range(ash::vk::WHOLE_SIZE)
            .build()
    }
}

impl<T> Drop for LveBuffer<T>
where
    T: PartialEq,
{
    fn drop(&mut self) {
        log::debug!("Dropping buffer");

        unsafe {
            self.unmap();
            self.lve_device.device.destroy_buffer(self.buffer, None);
            self.lve_device.device.free_memory(self.memory, None);
        }
    }
}