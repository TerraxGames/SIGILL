//! # Queue Family Abstractions
//! This module hosts basic abstractions for using queue families.

use std::collections::HashMap;

use ash::{prelude::VkResult, vk};

const GRAPHICS: &'static str = "graphics queue should be available";

#[derive(Debug)]
pub struct Queue {
    queue_info: (super::QueueFamilyIndex, super::QueueIndex),
    handle: Option<vk::Queue>,
    priority: f32,
}

impl Queue {
    pub fn new_empty(queue_info: (super::QueueFamilyIndex, super::QueueIndex), priority: f32) -> Self {
        Self {
            queue_info,
            handle: None,
            priority,
        }
    }

    pub fn populate_handle(&mut self, device: &super::Device) {
        self.handle = Some(device.get_device_queue(self.queue_info.0, self.queue_info.1));
    }

    #[inline]
    pub fn queue_info(&self) -> &(super::QueueFamilyIndex, super::QueueIndex) {
        &self.queue_info
    }

    #[inline]
    pub(super) fn handle(&self) -> vk::Queue {
        self.handle.expect("handle must be populated before being accessed")
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
    queue_priorities: HashMap<super::QueueFamilyIndex, Vec<f32>>,
}

impl QueueFamilies {
    pub fn new_empty(queue_family_map: &super::QueueFamilyMap) -> Self {
        let mut queues = HashMap::new();
        queues.insert(QueueType::Graphics, Queue::new_empty(*queue_family_map.get_queue_info(vk::QueueFlags::GRAPHICS).expect(GRAPHICS), 1.0));
        Self {
            queues,
            queue_priorities: HashMap::new(),
        }
    }

    #[inline]
    pub fn query_present_mode_queue(mut self, queue_family_map: &super::QueueFamilyMap, instance: &super::Instance, physical_device: vk::PhysicalDevice, surface: &super::Surface) -> VkResult<Self> {
        for (_, queue_info) in queue_family_map.inner().iter() {
            if instance.get_physical_device_surface_support(physical_device, queue_info.0, surface)? {
                self.queues.insert(QueueType::PresentMode, Queue::new_empty(*queue_info, 1.0));
            }
        }

        Ok(self)
    }

    pub fn populate_handles(&mut self, device: &super::Device) {
        self.queues.values_mut().for_each(|queue| queue.populate_handle(device));
    }

    pub fn get_queue_create_infos(&mut self, queue_family_map: &super::QueueFamilyMap) -> Vec<vk::DeviceQueueCreateInfo> {
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

    pub fn submit_queue<'a>(&self, device: &super::Device, queue_type: QueueType, submit: &'a vk::SubmitInfo2<'a>, fence: vk::Fence) -> VkResult<()> {
        device.submit_queue(self.get_queue(queue_type).handle.expect("queue must be initialized before being submitted"), submit, fence)
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
