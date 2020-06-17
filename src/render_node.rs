use crate::{
    shader::{FragmentShader, VertexShader},
    uniforms::UniformBindGroup,
};
use crate::{
    textures::TextureData,
    vertex_buffer::{VertexBuffer, VertexBufferData},
    GpuData, RenderError, textures::TextureShaderLayout,
};
use smallvec::SmallVec;
use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
    sync::Arc,
};

const VERTX_BUFFER_STACK_LIMIT: usize = 3;

pub struct RenderNode {
    shared_uniform_bind_groups: Vec<Arc<UniformBindGroup>>,
    local_uniform_bind_groups: Vec<UniformBindGroup>,
    vertex_buffer_types: SmallVec<[TypeId; VERTX_BUFFER_STACK_LIMIT]>,
    texture_types: Vec<TypeId>,
    pipeline: wgpu::RenderPipeline,
}

pub struct RenderNodeRunner<'a, 'b: 'a> {
    render_pass: wgpu::RenderPass<'a>,
    texture_types: &'b Vec<TypeId>,
    vertex_buffer_types: &'b SmallVec<[TypeId; VERTX_BUFFER_STACK_LIMIT]>,
    uniform_group_count: u32,
}

impl<'a, 'b: 'a> RenderNodeRunner<'a, 'b> {
    #[inline]
    pub fn set_texture_data<T: TextureShaderLayout>(&mut self, index: u32, data: &'b TextureData<T>) {
        assert!(
            TypeId::of::<T>() == self.texture_types[(index - self.uniform_group_count) as usize]
        );
        self.render_pass
            .set_bind_group(index, &data.bind_group, &[]);
    }

    #[inline]
    pub fn set_vertex_buffer_data<D: VertexBuffer>(
        &mut self,
        index: u32,
        data: &'b impl VertexBufferData<DataType = D>,
    ) {
        assert!(TypeId::of::<D>() == self.vertex_buffer_types[index as usize]);
        self.render_pass
            .set_vertex_buffer(index, data.get_gpu_buffer(), 0, 0);
    }
}

impl<'a> Deref for RenderNodeRunner<'a, '_> {
    type Target = wgpu::RenderPass<'a>;
    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl<'a> DerefMut for RenderNodeRunner<'a, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.render_pass
    }
}

#[derive(Default)]
pub struct RenderNodeBuilder<'a> {
    vertex_buffer_types: SmallVec<[TypeId; VERTX_BUFFER_STACK_LIMIT]>,
    vertex_buffer_descriptors:
        SmallVec<[wgpu::VertexBufferDescriptor<'a>; VERTX_BUFFER_STACK_LIMIT]>,
    local_uniform_bind_groups: Vec<UniformBindGroup>,
    shared_uniform_bind_groups: Vec<Arc<UniformBindGroup>>,
    vertex_shader: Option<VertexShader>,
    fragment_shader: Option<FragmentShader>,
    depth_stencil_desc: Option<wgpu::DepthStencilStateDescriptor>,
    rasterization_state_desc: Option<wgpu::RasterizationStateDescriptor>,
    texture_types: Vec<TypeId>,
    texture_layout_generators: Vec<Box<dyn Fn(&wgpu::Device) -> &'static wgpu::BindGroupLayout>>,
}

impl<'a> RenderNodeBuilder<'a> {
    pub fn add_vertex_buffer<VB: VertexBuffer>(mut self) -> Self {
        self.vertex_buffer_types.push(TypeId::of::<VB>());
        self.vertex_buffer_descriptors.push(VB::get_descriptor());
        self
    }

    pub fn add_local_uniform_bind_group(mut self, uniform: UniformBindGroup) -> Self {
        self.local_uniform_bind_groups.push(uniform);
        self
    }

    pub fn add_shared_uniform_bind_group(mut self, shared_uniform: Arc<UniformBindGroup>) -> Self {
        self.shared_uniform_bind_groups.push(shared_uniform);
        self
    }

    pub fn add_texture<T: TextureShaderLayout>(mut self) -> Self {
        self.texture_types.push(TypeId::of::<T>());
        self.texture_layout_generators
            .push(Box::new(move |device: &wgpu::Device| {
                T::get_layout(device)
            }));
        self
    }

    pub fn set_vertex_shader(mut self, vertex_shader: VertexShader) -> Self {
        self.vertex_shader = Some(vertex_shader);
        self
    }

    pub fn set_fragment_shader(mut self, fragment_shader: FragmentShader) -> Self {
        self.fragment_shader = Some(fragment_shader);
        self
    }

    pub fn set_default_rasterization_state(mut self) -> Self {
        self.rasterization_state_desc = Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        });
        self
    }

    pub fn set_rasterization_state(mut self, desc: wgpu::RasterizationStateDescriptor) -> Self {
        self.rasterization_state_desc = Some(desc);
        self
    }

    pub fn set_default_depth_stencil_state(mut self) -> Self {
        self.depth_stencil_desc = Some(wgpu::DepthStencilStateDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        });
        self
    }

    pub fn set_depth_stencil_state(mut self, desc: wgpu::DepthStencilStateDescriptor) -> Self {
        self.depth_stencil_desc = Some(desc);
        self
    }

    fn construct_pipeline(
        &mut self,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let texture_layouts = self
            .texture_layout_generators
            .iter()
            .map(|gen| gen(&device));

        let local_bind_group_layouts = self
            .local_uniform_bind_groups
            .iter()
            .map(UniformBindGroup::get_layout);

        let shared_bind_group_layouts = self
            .shared_uniform_bind_groups
            .iter()
            .map(|group| group.get_layout())
            .chain(local_bind_group_layouts)
            .chain(texture_layouts)
            .collect::<Vec<&wgpu::BindGroupLayout>>();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &shared_bind_group_layouts,
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: self.vertex_shader.as_ref().unwrap().get_descriptor(),
            fragment_stage: self
                .fragment_shader
                .as_ref()
                .map(FragmentShader::get_descriptor),
            rasterization_state: std::mem::replace(&mut self.rasterization_state_desc, None),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: color_format,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: std::mem::replace(&mut self.depth_stencil_desc, None),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &self.vertex_buffer_descriptors,
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }

    pub fn build(
        mut self,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
    ) -> Result<RenderNode, RenderError> {
        if self.vertex_shader.is_none() {
            Err(RenderError::MissingVertexShader)
        } else {
            let pipeline = self.construct_pipeline(device, color_format);
            Ok(RenderNode {
                shared_uniform_bind_groups: self.shared_uniform_bind_groups,
                local_uniform_bind_groups: self.local_uniform_bind_groups,
                pipeline,
                texture_types: self.texture_types,
                vertex_buffer_types: self.vertex_buffer_types,
            })
        }
    }
}

impl RenderNode {
    pub fn builder<'a>() -> RenderNodeBuilder<'a> {
        RenderNodeBuilder {
            vertex_buffer_types: SmallVec::new(),
            vertex_buffer_descriptors: SmallVec::new(),
            vertex_shader: None,
            fragment_shader: None,
            shared_uniform_bind_groups: Vec::new(),
            local_uniform_bind_groups: Vec::new(),
            rasterization_state_desc: None,
            depth_stencil_desc: None,
            texture_layout_generators: Vec::new(),
            texture_types: Vec::new(),
        }
    }

    #[inline]
    pub fn update(
        &self,
        device: &wgpu::Device,
        command_encoder: &mut wgpu::CommandEncoder,
        bind_group_index: usize,
        data: &impl GpuData,
    ) -> Result<(), RenderError> {
        self.local_uniform_bind_groups[bind_group_index].update_buffer_data(
            device,
            command_encoder,
            data,
        )
    }

    pub fn runner<'a: 'b, 'b>(
        &'a self,
        command_encoder: &'b mut wgpu::CommandEncoder,
        render_pass_descriptor: wgpu::RenderPassDescriptor<'b, '_>,
    ) -> RenderNodeRunner<'a, 'b> {
        let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&self.pipeline);
        let local_iter = self.local_uniform_bind_groups.iter();
        self.shared_uniform_bind_groups
            .iter()
            .map(|shared| shared.deref())
            .chain(local_iter)
            .enumerate()
            .for_each(|(i, group)| {
                render_pass.set_bind_group(i as u32, group.get_bind_group(), &[]);
            });

        RenderNodeRunner {
            render_pass,
            texture_types: &self.texture_types,
            vertex_buffer_types: &self.vertex_buffer_types,
            uniform_group_count: self.local_uniform_bind_groups.len() as u32,
        }
    }
}
