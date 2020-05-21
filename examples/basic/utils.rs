use super::Camera;
use crate::{to_vec, ModelInfo};
use smol_renderer::*;
#[repr(C)]
#[derive(GpuData)]
pub struct CameraGpuData {
    pub view_matrix: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_pos: [f32; 3],
}

impl From<Camera> for CameraGpuData {
    fn from(data: Camera) -> Self {
        let test = data.view_matrix.as_slice();
        let view_matrix = test
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();
        let projection = data
            .projection_matrix
            .as_matrix()
            .as_slice()
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();
        let view_pos = to_vec(&data.position);
        Self {
            view_matrix: [
                view_matrix[0],
                view_matrix[1],
                view_matrix[2],
                view_matrix[3],
            ],
            projection: [projection[0], projection[1], projection[2], projection[3]],
            view_pos: [view_pos.x, view_pos.y, view_pos.z],
        }
    }
}

impl From<ModelInfo> for RawModelInfo {
    fn from(data: ModelInfo) -> Self {
        let matrix = data
            .isometry
            .to_homogeneous()
            .as_slice()
            .chunks(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect::<Vec<[f32; 4]>>();
        RawModelInfo {
            model_matrix: [matrix[0], matrix[1], matrix[2], matrix[3]],
        }
    }
}
#[repr(C)]
#[derive(GpuData)]
pub struct RawModelInfo {
    pub model_matrix: [[f32; 4]; 4],
}

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub fn create_depth_texture(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::Texture {
    let desc = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device.create_texture(&desc)
}
#[repr(C)]
#[derive(GpuData)]
pub struct Vertex {
    pos: [f32; 3],
    tex_coord: [f32; 2],
}

fn vertex(position: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        pos: [position[0] as f32, position[1] as f32, position[2] as f32],
        tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}


impl VertexBufferData for Vertex {
    fn get_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float2],
        }
    }
}

pub struct Cube {
    vertices: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,
}

impl Drawable for Cube {
    fn draw<'b, 'a: 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, &self.vertices, 0, 0);
        render_pass.set_index_buffer(&self.index_buf, 0, 0);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

pub fn create_cube(device: &wgpu::Device) -> Cube {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];
    let bytes = vertex_data
        .iter()
        .map(GpuData::as_raw_bytes)
        .flatten()
        .copied()
        .collect::<Vec<u8>>();
    // index data format defaults to u32
    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];
    let index_count = index_data.len() as u32;
    let index_data = unsafe {
        std::slice::from_raw_parts(index_data.as_ptr() as *const u8, index_data.len() * 4)
    };
    Cube {
        vertices: device.create_buffer_with_data(bytes.as_slice(), wgpu::BufferUsage::VERTEX),
        index_buf: device.create_buffer_with_data(index_data, wgpu::BufferUsage::INDEX),
        index_count,
    }
}
