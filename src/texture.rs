use crate::RenderError;
use image::GenericImage;
use once_cell::sync::OnceCell;
use std::{marker::PhantomData, path::Path};

pub trait Texture: 'static {
    fn load(
        device: &wgpu::Device,
        path: impl AsRef<Path>,
        visibility: wgpu::ShaderStage,
    ) -> Result<(TextureData<Self>, wgpu::CommandBuffer), RenderError>
    where
        Self: std::marker::Sized;
    fn get_or_create_layout(
        device: &wgpu::Device,
        visibility: wgpu::ShaderStage,
    ) -> &'static wgpu::BindGroupLayout;
}

pub struct TextureData<T: Texture> {
    pub(crate) bind_group: wgpu::BindGroup,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    _marker: PhantomData<T>,
}

pub struct SimpleTexture;

impl Texture for SimpleTexture {
    fn get_or_create_layout(
        device: &wgpu::Device,
        visibility: wgpu::ShaderStage,
    ) -> &'static wgpu::BindGroupLayout {
        static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();
        LAYOUT.get_or_init(move || {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility,
                        ty: wgpu::BindingType::Sampler { comparison: true },
                    },
                ],
                label: None,
            })
        })
    }

    fn load(
        device: &wgpu::Device,
        path: impl AsRef<Path>,
        visibility: wgpu::ShaderStage,
    ) -> Result<(TextureData<SimpleTexture>, wgpu::CommandBuffer), RenderError> {
        let img = image::open(path)?;
        let img = img.flipv();

        let rgba = img.to_rgba(); // handle formats properly
        let (width, height) = img.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // handle formats properly
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        // Generate buffer + Bindbuffer + fill it with data
        let buffer = device.create_buffer_with_data(&rgba.to_vec(), wgpu::BufferUsage::COPY_SRC);

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture load encoder"),
        });
        // Encode a command that sends the data to the gpu so it can be bound to the texture in the shaders
        command_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * width,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            size,
        );
        // final buffer of the commands needed to send the texture to the GPU
        // So it can be used in the shaders
        let command_buffer = command_encoder.finish();

        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0, // related to mipmaps
            lod_max_clamp: 100.0,  // related to mipmaps
            compare: wgpu::CompareFunction::Always,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::get_or_create_layout(device, visibility),
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("SimpleTextureBindGroup"),
        });
        let texture_data = TextureData {
            bind_group,
            sampler,
            view,
            texture,
            _marker: PhantomData::default(),
        };
        Ok((texture_data, command_buffer))
    }
}
