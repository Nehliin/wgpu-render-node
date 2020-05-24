pub mod render_node;
pub mod shader;
pub mod texture;
pub mod uniforms;
pub mod vertex_buffer;

use std::path::PathBuf;
use thiserror::Error;

pub use vertex_buffer::{VertexBuffer, VertexData};
pub use render_node::{RenderNode, RenderNodeBuilder, RenderNodeRunner};
pub use shader::{FragmentShader, VertexShader};
pub use smol_renderer_derive::*;
pub use texture::{SimpleTexture, Texture};
pub use uniforms::{UniformBindGroup, UniformBindGroupBuilder};
pub unsafe trait GpuData: 'static + Sized {
    fn as_raw_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
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
