use crate::GpuData;
use std::marker::PhantomData;

pub struct VertexData<T> {
    pub(crate) buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

pub trait VertexBuffer: GpuData {
    fn allocate_buffer(device: &wgpu::Device, buffer_data: &[Self]) -> VertexData<Self> {
        let raw_bytes = buffer_data
            .iter()
            .map(GpuData::as_raw_bytes)
            .flatten()
            .copied()
            .collect::<Vec<u8>>();
        VertexData {
            _marker: PhantomData::default(),
            buffer: device.create_buffer_with_data(&raw_bytes, wgpu::BufferUsage::VERTEX),
        }
    }
    fn get_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}
