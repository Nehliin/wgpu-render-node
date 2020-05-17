use crate::{GpuData, RenderError};
use smallvec::SmallVec;
use std::any::TypeId;

const UNIFORM_STACK_LIMIT: usize = 5;
struct BindingInfo {
    size: usize,
    visibility: wgpu::ShaderStage,
}

pub struct UniformBindGroup {
    buffers: SmallVec<[(TypeId, wgpu::Buffer); UNIFORM_STACK_LIMIT]>,
    bind_group: Option<wgpu::BindGroup>, //Very ugly
    bind_group_layout: wgpu::BindGroupLayout,
}

impl UniformBindGroup {
    pub fn builder() -> UniformBindGroupBuilder {
        UniformBindGroupBuilder::new()
    }

    pub(crate) fn get_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub(crate) fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self
            .bind_group
            .as_ref()
            .expect("This should always be set in construction")
    }

    //TODO: a general Trait instead?
    pub fn update_buffer_data<T: GpuData>(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        data: &T,
    ) -> Result<(), RenderError> {
        if let Some((_, buffer)) = self.buffers.iter().find(|(id, _)| id == &TypeId::of::<T>()) {
            let staging_buffer =
                device.create_buffer_with_data(data.as_raw_bytes(), wgpu::BufferUsage::COPY_SRC);

            encoder.copy_buffer_to_buffer(
                &staging_buffer,
                0,
                &buffer,
                0,
                std::mem::size_of::<T>() as wgpu::BufferAddress,
            );
            Ok(())
        } else {
            Err(RenderError::GpuDataTypeNotPresent)
        }
    }
}

pub struct UniformBindGroupBuilder {
    builder_data: SmallVec<[(TypeId, BindingInfo); UNIFORM_STACK_LIMIT]>,
}

impl UniformBindGroupBuilder {
    fn new() -> Self {
        UniformBindGroupBuilder {
            builder_data: SmallVec::default(),
        }
    }

    pub fn add_binding<T: GpuData>(
        mut self,
        visibility: wgpu::ShaderStage,
    ) -> Result<Self, RenderError> {
        if self
            .builder_data
            .iter()
            .any(|(id, _)| id == &TypeId::of::<T>())
        {
            return Err(RenderError::GpuDataTypeAlreadyPresent);
        }
        if std::mem::size_of::<T>() == 0 {
            return Err(RenderError::ZeroSizedGpuData);
        }
        let binding_info = BindingInfo {
            size: std::mem::size_of::<T>(),
            visibility,
        };
        self.builder_data.push((TypeId::of::<T>(), binding_info));
        Ok(self)
    }

    pub fn build(self, device: &wgpu::Device) -> UniformBindGroup {
        let mut layout_entries: SmallVec<[wgpu::BindGroupLayoutEntry; UNIFORM_STACK_LIMIT]> =
            SmallVec::default();
        let mut buffers: SmallVec<[(TypeId, wgpu::Buffer); UNIFORM_STACK_LIMIT]> =
            SmallVec::default();
        for (i, (id, info)) in self.builder_data.iter().enumerate() {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("UniformBindingBuffer: {}", i)),
                size: info.size as u64,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

            buffers.push((*id, buffer));
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: info.visibility,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            })
        }

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &layout_entries,
            label: Some("UniformBindGroup Layout"),
        });
        let mut uniform_bind_group = UniformBindGroup {
            buffers,
            bind_group: None,
            bind_group_layout: layout,
        };
        {
            let mut bindings: SmallVec<[wgpu::Binding; UNIFORM_STACK_LIMIT]> = SmallVec::default();
            self.builder_data
                .iter()
                .enumerate()
                .zip(uniform_bind_group.buffers.iter())
                .for_each(|((i, (_, info)), (_, buffer))| {
                    bindings.push(wgpu::Binding {
                        binding: i as u32,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &buffer,
                            range: 0..info.size as wgpu::BufferAddress,
                        },
                    });
                });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group.bind_group_layout,
                bindings: &bindings,
                label: Some("UniformBindGroup"),
            });
            uniform_bind_group.bind_group = Some(bind_group)
        }
        uniform_bind_group
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_test_env() -> (wgpu::Device, wgpu::Queue) {
        futures::executor::block_on(async {
            let adapter = wgpu::Adapter::request(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: None,
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();
            adapter
                .request_device(&wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: Default::default(),
                })
                .await
        })
    }
    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct Data1 {
        dummy: i32,
    }
    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct Data2 {
        dummy: i32,
    }
    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct Data3 {
        dummy: i32,
    }

    unsafe impl GpuData for Data1 {
        fn as_raw_bytes(&self) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(
                    self as *const Data1 as *const u8,
                    std::mem::size_of::<Self>(),
                )
            }
        }
    }

    unsafe impl GpuData for Data2 {
        fn as_raw_bytes(&self) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(
                    self as *const Data2 as *const u8,
                    std::mem::size_of::<Self>(),
                )
            }
        }
    }

    unsafe impl GpuData for Data3 {
        fn as_raw_bytes(&self) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(
                    self as *const Data3 as *const u8,
                    std::mem::size_of::<Self>(),
                )
            }
        }
    }

    #[test]
    fn construction() -> Result<(), RenderError> {
        let (device, _) = create_test_env();
        let group = UniformBindGroup::builder()
            .add_binding::<Data1>(wgpu::ShaderStage::VERTEX)?
            .add_binding::<Data2>(wgpu::ShaderStage::FRAGMENT)?
            .build(&device);

        let data1 = Data1 { dummy: 1 };

        let data2 = Data2 { dummy: 2 };
        let data3 = Data3 { dummy: 3 };

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        assert!(group
            .update_buffer_data(&device, &mut encoder, &data1)
            .is_ok());
        assert!(group
            .update_buffer_data(&device, &mut encoder, &data2)
            .is_ok());
        assert!(group
            .update_buffer_data(&device, &mut encoder, &data3)
            .is_err());
        Ok(())
    }
}
