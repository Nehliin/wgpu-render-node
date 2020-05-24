use crate::{
    shader::{FragmentShader, VertexShader},
    uniforms::UniformBindGroup,
};
use crate::{
    texture::TextureData,
    vertex_buffer::{VertexBuffer, ImmutableVertexData, MutableVertexData, VertexBufferData},
    GpuData, RenderError, Texture,
};
use smallvec::SmallVec;
use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

const VERTX_BUFFER_STACK_LIMIT: usize = 3;

pub struct RenderNode {
    uniform_bind_groups: Vec<UniformBindGroup>,
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
    pub fn set_texture_data<T: Texture>(&mut self, index: u32, data: &'b TextureData<T>) {
        assert!(
            TypeId::of::<T>() == self.texture_types[(index - self.uniform_group_count) as usize]
        );
        self.render_pass
            .set_bind_group(index, &data.bind_group, &[]);
    }

    #[inline]
    pub fn set_vertex_buffer_data<D: VertexBuffer>(&mut self, index: u32, data: &'b impl VertexBufferData<DataType = D>) {
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
    uniform_bind_groups: Vec<UniformBindGroup>,
    vertex_shader: Option<VertexShader>,
    fragment_shader: Option<FragmentShader>,
    texture_types: Vec<TypeId>,
    texture_layout_generators: Vec<Box<dyn Fn(&wgpu::Device) -> &'static wgpu::BindGroupLayout>>,
}

impl<'a> RenderNodeBuilder<'a> {
    pub fn add_vertex_buffer<VB: VertexBuffer>(mut self) -> Self {
        self.vertex_buffer_types.push(TypeId::of::<VB>());
        self.vertex_buffer_descriptors.push(VB::get_descriptor());
        self
    }

    pub fn add_uniform_bind_group(mut self, uniform: UniformBindGroup) -> Self {
        self.uniform_bind_groups.push(uniform);
        self
    }

    pub fn add_texture<T: Texture>(mut self, visibility: wgpu::ShaderStage) -> Self {
        self.texture_types.push(TypeId::of::<T>());
        self.texture_layout_generators
            .push(Box::new(move |device: &wgpu::Device| {
                T::get_or_create_layout(device, visibility)
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

    fn construct_pipeline(
        &self,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let texture_layouts = self
            .texture_layout_generators
            .iter()
            .map(|gen| gen(&device));

        let bind_group_layouts = self
            .uniform_bind_groups
            .iter()
            .map(UniformBindGroup::get_layout)
            .chain(texture_layouts)
            .collect::<Vec<&wgpu::BindGroupLayout>>();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &bind_group_layouts,
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: self.vertex_shader.as_ref().unwrap().get_descriptor(),
            fragment_stage: self
                .fragment_shader
                .as_ref()
                .map(FragmentShader::get_descriptor),
            // TODO: add customizable rasterization stage
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: color_format,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
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
        self,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Result<RenderNode, RenderError> {
        if self.vertex_shader.is_none() {
            Err(RenderError::MissingVertexShader)
        } else {
            let pipeline = self.construct_pipeline(device, color_format, depth_format);
            Ok(RenderNode {
                uniform_bind_groups: self.uniform_bind_groups,
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
            uniform_bind_groups: Vec::new(),
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
        self.uniform_bind_groups[bind_group_index].update_buffer_data(device, command_encoder, data)
    }

    pub fn runner<'a: 'b, 'b>(
        &'a self,
        command_encoder: &'b mut wgpu::CommandEncoder,
        render_pass_descriptor: wgpu::RenderPassDescriptor<'b, '_>,
    ) -> RenderNodeRunner<'a, 'b> {
        let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&self.pipeline);
        self.uniform_bind_groups
            .iter()
            .enumerate()
            .for_each(|(i, group)| {
                render_pass.set_bind_group(i as u32, group.get_bind_group(), &[]);
            });

        RenderNodeRunner {
            render_pass,
            texture_types: &self.texture_types,
            vertex_buffer_types: &self.vertex_buffer_types,
            uniform_group_count: self.uniform_bind_groups.len() as u32,
        }
    }
}
