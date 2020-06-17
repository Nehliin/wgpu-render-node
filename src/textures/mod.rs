use crate::RenderError;
use std::{marker::PhantomData, path::Path};

pub mod simpletexture;

pub trait TextureShaderLayout: 'static {
    const VISIBILITY: wgpu::ShaderStage;
    fn get_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout;
}

pub struct TextureData<T: TextureShaderLayout> {
    _marker: PhantomData<T>,
    pub bind_group: wgpu::BindGroup,
    pub views: Vec<wgpu::TextureView>,
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
}

impl<T: TextureShaderLayout> TextureData<T> {
    pub fn new(
        bind_group: wgpu::BindGroup,
        texture: wgpu::Texture,
        views: Vec<wgpu::TextureView>,
        sampler: wgpu::Sampler,
    ) -> Self {
        TextureData {
            bind_group,
            texture,
            views,
            sampler,
            _marker: PhantomData::default(),
        }
    }
}

pub trait LoadableTexture: Sized + TextureShaderLayout {
    fn load_texture(
        device: &wgpu::Device,
        path: impl AsRef<Path>,
    ) -> Result<(TextureData<Self>, wgpu::CommandBuffer), RenderError>;
}

pub trait RenderTargetTexture: Sized {
    fn allocate_texture(device: &wgpu::Device) -> TextureData<Self> where Self: TextureShaderLayout;
}
