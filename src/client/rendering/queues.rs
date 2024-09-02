//! # Queue Family Abstractions
//! This module hosts basic abstractions for using queue families.

use std::collections::HashMap;

use ash::{prelude::VkResult, vk};

use super::vulkan;

const GRAPHICS: &'static str = "graphics queue should be available";

#[derive(Debug)]
pub struct Queue {
    queue_info: (vulkan::QueueFamilyIndex, vulkan::QueueIndex),
    handle: Option<vk::Queue>,
    priority: f32,
}

impl Queue {
    pub fn new_empty(queue_info: (vulkan::QueueFamilyIndex, vulkan::QueueIndex), priority: f32) -> Self {
        Self {
            queue_info,
            handle: None,
            priority,
        }
    }

    pub fn populate_handle(&mut self, device: &vulkan::Device) {
        self.handle = Some(device.get_device_queue(self.queue_info.0, self.queue_info.1));
    }

    #[inline]
    pub fn queue_info(&self) -> &(vulkan::QueueFamilyIndex, vulkan::QueueIndex) {
        &self.queue_info
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub enum QueueType {
    Graphics,
    PresentMode,
}

#[derive(Debug)]
pub struct QueueFamilies {
    queues: HashMap<QueueType, Queue>,
    queue_priorities: HashMap<vulkan::QueueFamilyIndex, Vec<f32>>,
}

impl QueueFamilies {
    pub fn new_empty(queue_family_map: &vulkan::QueueFamilyMap) -> Self {
        let mut queues = HashMap::new();
        queues.insert(QueueType::Graphics, Queue::new_empty(*queue_family_map.get_queue_info(vk::QueueFlags::GRAPHICS).expect(GRAPHICS), 1.0));
        Self {
            queues,
            queue_priorities: HashMap::new(),
        }
    }

    #[inline]
    pub fn query_present_mode_queue(mut self, queue_family_map: &vulkan::QueueFamilyMap, instance: &vulkan::Instance, physical_device: vk::PhysicalDevice, surface: &vulkan::Surface) -> VkResult<Self> {
        for (_, queue_info) in queue_family_map.inner().iter() {
            if instance.get_physical_device_surface_support(physical_device, queue_info.0, surface)? {
                self.queues.insert(QueueType::PresentMode, Queue::new_empty(*queue_info, 1.0));
            }
        }

        Ok(self)
    }

    pub fn populate_handles(&mut self, device: &vulkan::Device) {
        self.queues.values_mut().for_each(|queue| queue.populate_handle(device));
    }

    pub fn get_queue_create_infos(&mut self, queue_family_map: &vulkan::QueueFamilyMap) -> Vec<vk::DeviceQueueCreateInfo> {
        // Detect queue families and map them to their length.
        let mut family2len_map = HashMap::new();
        for (_, (queue_family_index, queue_index)) in queue_family_map.inner().iter() {
            if !family2len_map.contains_key(queue_family_index) || family2len_map.get(queue_family_index).unwrap() - 1 < *queue_index {
                family2len_map.insert(*queue_family_index, queue_index + 1);
            }
        }

        // Map the queue families to their queues' priorities.
        let family2priority_map = &mut self.queue_priorities;
        for (queue_family_index, queue_family_length) in family2len_map {
            let mut priorities = vec![0.0f32; queue_family_length as usize];
            for (_, queue) in self.queues.iter() {
                if queue.queue_info.0 == queue_family_index {
                    let _ = std::mem::replace(&mut priorities[queue.queue_info.1 as usize], queue.priority);
                }
            }
            family2priority_map.insert(queue_family_index, priorities);
        }

        // Fabricate create info for the queue families from their index and length.
        let mut create_infos = Vec::new();
        for (queue_family_index, priorities) in family2priority_map.iter() {
            create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*queue_family_index)
                    .queue_priorities(priorities.as_slice())
            );
        }

        create_infos
    }

    fn get_queue(&self, queue_type: QueueType) -> &Queue {
        self.queues.get(&queue_type).unwrap()
    }

    #[inline]
    pub fn graphics(&self) -> &Queue {
        self.get_queue(QueueType::Graphics)
    }

    #[inline]
    pub fn present_mode(&self) -> &Queue {
        self.get_queue(QueueType::PresentMode)
    }
}
