use crate::GpuData;
use std::marker::PhantomData;

pub trait VertexBufferData {
    type DataType: VertexBuffer;
    fn get_gpu_buffer(&self) -> &wgpu::Buffer;
}

pub struct ImmutableVertexData<T: GpuData> {
    pub(crate) buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

pub struct MutableVertexData<T: GpuData> {
    pub(crate) buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T: VertexBuffer> VertexBufferData for ImmutableVertexData<T> {
    type DataType = T;

    fn get_gpu_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl<T: VertexBuffer> VertexBufferData for MutableVertexData<T> {
    type DataType = T;

    fn get_gpu_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl<T: VertexBuffer> MutableVertexData<T> {
    #[allow(dead_code)]
    pub fn update(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        buffer_data: &[T],
    ) {
        let raw_bytes = buffer_data
            .iter()
            .map(GpuData::as_raw_bytes)
            .flatten()
            .copied()
            .collect::<Vec<u8>>();

        let staging_buffer =
            device.create_buffer_with_data(&raw_bytes, wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, raw_bytes.len() as u64);
    }
}

pub trait VertexBuffer: GpuData {
    const STEP_MODE: wgpu::InputStepMode;

    fn allocate_immutable_buffer(
        device: &wgpu::Device,
        buffer_data: &[Self],
    ) -> ImmutableVertexData<Self> {
        let raw_bytes = buffer_data
            .iter()
            .map(GpuData::as_raw_bytes)
            .flatten()
            .copied()
            .collect::<Vec<u8>>();
        ImmutableVertexData {
            _marker: PhantomData::default(),
            buffer: device.create_buffer_with_data(&raw_bytes, wgpu::BufferUsage::VERTEX),
        }
    }

    fn allocate_mutable_buffer(
        device: &wgpu::Device,
        buffer_data: &[Self],
    ) -> MutableVertexData<Self> {
        let raw_bytes = buffer_data
            .iter()
            .map(GpuData::as_raw_bytes)
            .flatten()
            .copied()
            .collect::<Vec<u8>>();
        MutableVertexData {
            _marker: PhantomData::default(),
            buffer: device.create_buffer_with_data(
                &raw_bytes,
                wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            ),
        }
    }

    fn get_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: Self::STEP_MODE,
            attributes: Self::get_attributes(),
        }
    }

    fn get_attributes<'a>() -> &'a [wgpu::VertexAttributeDescriptor];
}
