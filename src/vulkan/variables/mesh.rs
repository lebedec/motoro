use crate::math::{Vec2, Vec4, VecArith, VecComponents};
use crate::vulkan::{
    create_buffers, get_memory_type_index, MemoryBuffer, Vulkan,
};
use crate::Colors;


use vulkanalia::vk::{
    BufferCreateInfo, BufferUsageFlags, DeviceV1_0, Format, HasBuilder,
    InstanceV1_0, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, PhysicalDevice,
    PipelineVertexInputStateCreateInfo, SharingMode,
    VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
};
use vulkanalia::{Device, Instance};

/// Represents GLSL vertices static buffer.
pub struct Mesh {
    pub buffers: Vec<MemoryBuffer>,
    device: Device,
    pub vertices: Vec<Vertex>,
    pub cursor: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Vertices {
    pub ptr: usize,
    pub len: usize,
}

impl Mesh {
    pub const RECT_VERTICES_N: usize = 6;

    pub fn add_rect(&mut self, start: Vec2, size: Vec2, color: impl Colors) -> Option<Vertices> {
        let a = start;
        let b = start.add([size.x(), 0.0]);
        let c = start.add(size);
        let d = start.add([0.0, size.y()]);
        self.add_polygon(&[a, b, c, d], color)
    }

    pub fn add_polygon(&mut self, vertices: &[Vec2], color: impl Colors) -> Option<Vertices> {
        let color = color.to_vec4();
        let mut vertices: Vec<Vertex> = vertices
            .iter()
            .map(|position| Vertex {
                position: *position,
                color,
                uv: [0.0, 0.0],
            })
            .collect();
        if vertices.len() > 3 {
            // default renderer uses TRIANGLES_LIST mode
            let mut triangles = vec![];
            for n in 2..vertices.len() {
                triangles.push(vertices[0]);
                triangles.push(vertices[n - 1]);
                triangles.push(vertices[n])
            }
            vertices = triangles;
        }
        self.append(&vertices)
    }

    pub unsafe fn create(vulkan: &Vulkan, n: usize) -> Self {
        let device = vulkan.device.clone();
        let frames = vulkan.swapchain.images.len();
        let physical_device_memory = vulkan
            .instance
            .get_physical_device_memory_properties(vulkan.physical_device);
        let buffers = create_buffers(
            BufferUsageFlags::VERTEX_BUFFER,
            &device,
            frames,
            physical_device_memory,
            n * std::mem::size_of::<Vertex>(),
        );
        let vertices = vec![Vertex::default(); n];
        Self {
            buffers,
            device,
            vertices,
            cursor: 0,
        }
    }

    pub fn input_state(&self) -> Option<PipelineVertexInputStateCreateInfo> {
        Some(Vertex::input_state())
    }

    pub fn append(&mut self, vertices: &[Vertex]) -> Option<Vertices> {
        let ptr = self.cursor;
        let len = vertices.len();
        if ptr + len > self.vertices.len() {
            return None;
        }
        self.vertices[ptr..ptr + len].copy_from_slice(vertices);
        self.cursor = ptr + len;
        Some(Vertices { ptr, len })
    }

    pub fn update_all(&mut self) {
        // TODO: single buffer, no swapchain
        for chain in 0..self.buffers.len() {
            self.update(chain);
        }
    }

    pub fn update(&mut self, frame: usize) -> usize {
        let value = self.vertices.as_slice();
        let count = self.cursor;
        self.cursor = 0;
        self.update_from(frame, value);
        count
    }

    pub fn update_from(&self, frame: usize, value: &[Vertex]) {
        unsafe {
            let memory = self
                .device
                .map_memory(
                    self.buffers[frame].memory,
                    0,
                    (value.len() * std::mem::size_of::<Vertex>()) as u64,
                    MemoryMapFlags::empty(),
                )
                .expect("memory must be mapped");
            std::ptr::copy_nonoverlapping(value.as_ptr(), memory.cast(), value.len());
            self.device.unmap_memory(self.buffers[frame].memory);
        }
    }

    pub fn destroy(&self) {
        for buffer in &self.buffers {
            buffer.destroy(&self.device);
        }
    }
}

pub unsafe fn create_vertex_buffer(
    device: &Device,
    instance: &Instance,
    physical_device: PhysicalDevice,
    vertices: &[Vertex],
) -> MemoryBuffer {
    let buffer_info = BufferCreateInfo::builder()
        .size((std::mem::size_of::<Vertex>() * vertices.len()) as u64)
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
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Vec4,
    pub uv: Vec2,
}

impl Vertex {
    pub const RECTANGLE: [Vertex; 6] = [
        Vertex {
            position: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5],
            color: [0.0, 1.0, 0.0, 1.0],
            uv: [1.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
            uv: [0.0, 1.0],
        },
        Vertex {
            position: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0, 1.0],
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
            format: Format::R32G32B32A32_SFLOAT,
            offset: 8,
        },
        VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: Format::R32G32_SFLOAT,
            offset: 24,
        },
    ];

    const BINDING: [VertexInputBindingDescription; 1] = [VertexInputBindingDescription {
        binding: 0,
        stride: 32,
        input_rate: VertexInputRate::VERTEX,
    }];

    pub fn input_state() -> PipelineVertexInputStateCreateInfo {
        PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&Self::BINDING)
            .vertex_attribute_descriptions(&Self::ATTRIBUTES)
            .build()
    }
}
