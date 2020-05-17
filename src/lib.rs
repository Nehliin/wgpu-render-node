pub mod render_node;
pub mod shader;
pub mod texture;
pub mod uniforms;
pub mod vertex_buffers;

use std::path::PathBuf;
use thiserror::Error;

pub use render_node::{RenderNode, RenderNodeBuilder};
pub use shader::{FragmentShader, FragmentShaderBuilder, VertexShader, VertexShaderBuilder};
pub use texture::{SimpleTexture, Texture};
pub use uniforms::{UniformBindGroup, UniformBindGroupBuilder};
pub unsafe trait GpuData: 'static {
    fn as_raw_bytes(&self) -> &[u8];
}
// TODO: Add index format associated type to this trait
pub trait VertexBufferData: GpuData {
    fn get_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Couldn't compile shader file {path:?}: {compile_error:?}")]
    ShaderCompileError {
        compile_error: String,
        path: PathBuf,
    },

    #[error("You must set a VertexShader")]
    MissingVertexShader,

    #[error("Couldn't open image")]
    TextureLoadError(#[from] image::ImageError),

    #[error("Issue with shader file")]
    ShaderFileError(#[from] std::io::Error),

    #[error("GpuData can't be zero sized")]
    ZeroSizedGpuData,

    #[error("There is already a binding for this GpuData in this bindgroup")]
    GpuDataTypeAlreadyPresent,

    #[error("There doesn't exist a binding for this GpuData in this bindgroup")]
    GpuDataTypeNotPresent,
}
