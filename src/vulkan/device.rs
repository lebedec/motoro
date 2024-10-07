use std::collections::HashMap;
use std::env;
use std::ffi::c_char;

use log::info;
use vulkanalia::vk::{HasBuilder, InstanceV1_0, InstanceV1_1};
use vulkanalia::{vk, Device, Instance};

use crate::vulkan::{QueueFamilyIndex, DEVICE_EXTENSIONS, VALIDATION_LAYER};

pub unsafe fn create_logical_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    queues: QueueFamilyIndex,
) -> Device {
    let mut priority = HashMap::new();
    for index in queues.indices() {
        let queue_priorities = vec![1.0; (index.queue + 1) as usize];
        priority.insert(index.family, queue_priorities);
    }
    let queue_infos: Vec<_> = priority
        .iter()
        .map(|(family, queue_priorities)| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*family)
                .queue_priorities(queue_priorities)
        })
        .collect();
    let mut layers = vec![];
    let is_vulkan_debug = env::var("VULKAN_DEBUG").is_ok();
    if is_vulkan_debug {
        info!("Enables device validation layer");
        layers.push(VALIDATION_LAYER.as_ptr());
    }
    info!("Extensions: {:?}", DEVICE_EXTENSIONS);

    // see vulkanalia chain.rs for details about pointer chain push_next
    let mut indexing = vk::PhysicalDeviceDescriptorIndexingFeatures::default();
    let mut features2 = vk::PhysicalDeviceFeatures2::builder().push_next(&mut indexing);
    instance.get_physical_device_features2(physical_device, &mut features2);
    info!(
        "shaderSampledImageArrayNonUniformIndexing: {}",
        indexing.shader_sampled_image_array_non_uniform_indexing
    );
    info!(
        "descriptorBindingSampledImageUpdateAfterBind: {}",
        indexing.descriptor_binding_sampled_image_update_after_bind
    );
    info!(
        "shaderUniformBufferArrayNonUniformIndexing: {}",
        indexing.shader_uniform_buffer_array_non_uniform_indexing
    );
    info!(
        "descriptorBindingUniformBufferUpdateAfterBind: {}",
        indexing.descriptor_binding_uniform_buffer_update_after_bind,
    );
    info!(
        "shaderStorageBufferArrayNonUniformIndexing: {}",
        indexing.shader_storage_buffer_array_non_uniform_indexing
    );
    info!(
        "descriptorBindingStorageBufferUpdateAfterBind: {}",
        indexing.descriptor_binding_storage_buffer_update_after_bind,
    );
    info!(
        "Runtime descriptor array: {}",
        indexing.runtime_descriptor_array
    );
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    info!(
        "Min storage buffer offset alignment: {}",
        properties.limits.min_storage_buffer_offset_alignment
    );
    info!(
        "Max storage buffer range: {}",
        properties.limits.max_storage_buffer_range
    );
    info!(
        "Max descriptor set storage buffers: {}",
        properties.limits.max_descriptor_set_storage_buffers
    );
    info!(
        "Max descriptor set storage buffers dynamic: {}",
        properties.limits.max_descriptor_set_storage_buffers_dynamic
    );
    info!(
        "Min uniform buffer offset alignment: {}",
        properties.limits.min_uniform_buffer_offset_alignment
    );
    // let mut features12: vk::PhysicalDeviceVulkan12Features = unsafe { std::mem::zeroed() };
    // features12.shader_sampled_image_array_non_uniform_indexing = 1;
    //
    //
    let mut indexing = vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
        // Enable non sized arrays
        .runtime_descriptor_array(true)
        // Enable non bound variables slots
        .descriptor_binding_partially_bound(true)
        // Enable non uniform array indexing
        // (#extension GL_EXT_nonuniform_qualifier : require)
        //.shader_storage_buffer_array_non_uniform_indexing(true)
        .shader_sampled_image_array_non_uniform_indexing(true)
        //.shader_storage_image_array_non_uniform_indexing(true)
        // All of these enables to update after the
        // command buffer used the bindDescriptorsSet
        //.descriptor_binding_storage_buffer_update_after_bind(true)
        .descriptor_binding_sampled_image_update_after_bind(true)
        //.descriptor_binding_storage_image_update_after_bind(true)
        //.descriptor_binding_uniform_buffer_update_after_bind(true);
        ;

    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .fill_mode_non_solid(true);

    let extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();
    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features)
        .push_next(&mut indexing);
    // .push_next(&mut features12);
    info!("Creates Vulkan logical device");
    instance
        .create_device(physical_device, &info, None)
        .expect("Vulkan device must be created")
}
