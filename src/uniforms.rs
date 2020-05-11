use smallvec::SmallVec;
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

pub trait Raw {
    fn as_raw_bytes(&self) -> &[u8];
}

const UNIFORM_STACK_LIMIT: usize = 5;
struct BindingInfo {
    size: usize,
    visibility: wgpu::ShaderStage,
}

pub struct UniformBindGroup {
    buffers: SmallVec<[(TypeId, wgpu::Buffer); UNIFORM_STACK_LIMIT]>,
    bind_group: Option<wgpu::BindGroup>,
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
    pub fn upload<T: Raw + 'static>(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        data: &T,
    ) {
        if let Some((_, buffer)) = self.buffers.iter().find(|(id, _)| id == &TypeId::of::<T>()) {
            let staging_buffer =
                device.create_buffer_with_data(data.as_raw_bytes(), wgpu::BufferUsage::COPY_SRC);

            encoder.copy_buffer_to_buffer(
                &staging_buffer,
                0,
                &buffer,
                0,
                std::mem::size_of::<T>() as wgpu::BufferAddress,
            )
        } else {
            println!("No such uniform binding exists {:?}", TypeId::of::<T>());
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

    pub fn add_binding<T: Raw + 'static>(mut self, visibility: wgpu::ShaderStage) -> Self {
        if let Some((id, _)) = self
            .builder_data
            .iter()
            .find(|(id, _)| id == &TypeId::of::<T>())
        {
            println!("{:?}, already added as a binding", id);
            return self;
        }
        let binding_info = BindingInfo {
            size: std::mem::size_of::<T>(),
            visibility,
        };
        self.builder_data.push((TypeId::of::<T>(), binding_info));
        self
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

    struct Data1;
    struct Data2;

    impl Raw for Data1 {
        fn as_raw_bytes(&self) -> &[u8] {
            &[5, 0]
        }
    }

    impl Raw for Data2 {
        fn as_raw_bytes(&self) -> &[u8] {
            &[5, 0]
        }
    }

    #[test]
    fn construction() {
        let test = UniformBindGroup::builder()
            .add_binding::<Data1>(wgpu::ShaderStage::VERTEX)
            .add_binding::<Data2>(wgpu::ShaderStage::FRAGMENT)
            .build();

        let expected_layout = &wgpu::BindGroupLayout {}
        test.get_layout();
    }
}
