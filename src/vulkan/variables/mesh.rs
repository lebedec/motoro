use vulkanalia::vk::{
    BufferCreateInfo, BufferUsageFlags, DeviceV1_0, Format, HasBuilder, InstanceV1_0,
    MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, PhysicalDevice,
    PipelineVertexInputStateCreateInfo, SharingMode, VertexInputAttributeDescription,
    VertexInputBindingDescription, VertexInputRate,
};
use vulkanalia::{Device, Instance};

use crate::math::{Vec2, Vec3};
use crate::vulkan::{get_memory_type_index, MemoryBuffer, Vulkan};

/// Represents GLSL vertices static buffer.
pub struct Mesh {
    pub(crate) buffer: MemoryBuffer,
}

impl Mesh {
    pub unsafe fn create(vertices: &[Vertex2D], vulkan: &Vulkan) -> Self {
        let buffer = create_vertex_buffer(
            &vulkan.device,
            &vulkan.instance,
            vulkan.physical_device,
            vertices,
        );
        Self { buffer }
    }
}

pub unsafe fn create_vertex_buffer(
    device: &Device,
    instance: &Instance,
    physical_device: PhysicalDevice,
    vertices: &[Vertex2D],
) -> MemoryBuffer {
    let buffer_info = BufferCreateInfo::builder()
        .size((std::mem::size_of::<Vertex2D>() * vertices.len()) as u64)
        .usage(BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(SharingMode::EXCLUSIVE);
    let handle = device
        .create_buffer(&buffer_info, None)
        .expect("buffer must be created");

    let requirements = device.get_buffer_memory_requirements(handle);
    let physical_device_memory = instance.get_physical_device_memory_properties(physical_device);
    let memory_type_index = get_memory_type_index(
        MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE,
        requirements,
        physical_device_memory,
    );
    let memory_info = MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);
    let memory = device
        .allocate_memory(&memory_info, None)
        .expect("vertex buffer memory must be allocated");

    device
        .bind_buffer_memory(handle, memory, 0)
        .expect("vertex buffer must bound");
    let pointer = device
        .map_memory(memory, 0, buffer_info.size, MemoryMapFlags::empty())
        .expect("vertex buffer memory must be mapped");
    std::ptr::copy_nonoverlapping(vertices.as_ptr(), pointer.cast(), vertices.len());
    device.unmap_memory(memory);

    MemoryBuffer { handle, memory }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex2D {
    pub position: Vec2,
    pub color: Vec3,
    pub uv: Vec2,
}

impl Vertex2D {
    pub const RECTANGLE: [Vertex2D; 6] = [
        Vertex2D {
            position: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0],
            uv: [0.0, 0.0],
        },
        Vertex2D {
            position: [0.5, -0.5],
            color: [0.0, 1.0, 0.0],
            uv: [1.0, 0.0],
        },
        Vertex2D {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0],
            uv: [1.0, 1.0],
        },
        Vertex2D {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0],
            uv: [1.0, 1.0],
        },
        Vertex2D {
            position: [-0.5, 0.5],
            color: [0.0, 0.0, 1.0],
            uv: [0.0, 1.0],
        },
        Vertex2D {
            position: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0],
            uv: [0.0, 0.0],
        },
    ];

    const ATTRIBUTES: [VertexInputAttributeDescription; 3] = [
        VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: Format::R32G32_SFLOAT,
            offset: 0,
        },
        VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: Format::R32G32B32_SFLOAT,
            offset: 8,
        },
        VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: Format::R32G32_SFLOAT,
            offset: 20,
        },
    ];

    const BINDING: [VertexInputBindingDescription; 1] = [VertexInputBindingDescription {
        binding: 0,
        stride: 28,
        input_rate: VertexInputRate::VERTEX,
    }];

    pub fn input_state() -> PipelineVertexInputStateCreateInfo {
        PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&Self::BINDING)
            .vertex_attribute_descriptions(&Self::ATTRIBUTES)
            .build()
    }

    pub fn no_input() -> PipelineVertexInputStateCreateInfo {
        PipelineVertexInputStateCreateInfo::builder().build()
    }
}
