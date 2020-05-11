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
    bind_group: wgpu::BindGroup,
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
        &self.bind_group
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

    pub fn add_binding<T: Raw + 'static>(&mut self, visibility: wgpu::ShaderStage) {
        if let Some((id, _)) = self
            .builder_data
            .iter()
            .find(|(id, _)| id == &TypeId::of::<T>())
        {
            println!("{:?}, already added as a binding", id);
            return;
        }
        let binding_info = BindingInfo {
            size: std::mem::size_of::<T>(),
            visibility,
        };
        self.builder_data.push((TypeId::of::<T>(), binding_info));
    }

    pub fn build(self, device: &wgpu::Device) -> UniformBindGroup {
        let mut layout_entries: SmallVec<[wgpu::BindGroupLayoutEntry; UNIFORM_STACK_LIMIT]> =
            SmallVec::default();
        let mut buffers: SmallVec<[(TypeId, wgpu::Buffer); UNIFORM_STACK_LIMIT]> =
            SmallVec::default();
        let mut bindings: SmallVec<[wgpu::Binding; UNIFORM_STACK_LIMIT]> = SmallVec::default();

        for (i, (id, info)) in self.builder_data.into_iter().enumerate() {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("UniformBindingBuffer: {}", i)),
                size: info.size as u64,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

            buffers.push((id, buffer));
            let buffer = &buffers[buffers.len() - 1].1;
            bindings.push(wgpu::Binding {
                binding: i as u32,
                resource: wgpu::BindingResource::Buffer {
                    buffer,
                    range: 0..info.size as wgpu::BufferAddress,
                },
            });
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: info.visibility,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            })
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &layout_entries,
            label: Some("UniformBindGroup Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &bindings,
            label: Some("UniformBindGroup"),
        });

        UniformBindGroup {
            buffers,
            bind_group,
            bind_group_layout,
        }
    }
}
// might not be needed after all?
/*pub struct Uniform<T: Raw> {
    _marker: PhantomData<T>,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

pub struct UniformStorage {
    storage: SmallVec<[(TypeId, Box<dyn Any>); 7]>,
}

impl UniformStorage {
    fn new() -> Self {
        Self {
            storage: SmallVec::default(),
        }
    }

    fn set_uniform<U: 'static>(&mut self, uniform: U) {
        if let Some((id, _)) = self.storage.iter().find(|(id, _)| id == &TypeId::of::<U>()) {
            println!("{:?}, allready added as a uniform", id);
            return;
        }
        self.storage.push((TypeId::of::<U>(), Box::new(uniform)));
    }

    fn get_uniform<U: 'static>(&self) -> Option<&U> {
        self.storage
            .iter()
            .find(|(id, _)| id == &TypeId::of::<U>())
            .map(|(_, boxed_val)| boxed_val.downcast_ref::<U>().unwrap())
    }

    fn get_uniform_mut<U: 'static>(&mut self) -> Option<&mut U> {
        self.storage
            .iter_mut()
            .find(|(id, _)| id == &TypeId::of::<U>())
            .map(|(_, boxed_val)| boxed_val.downcast_mut::<U>().unwrap())
    }
}*/
