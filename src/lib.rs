pub mod shader;
pub mod uniforms;
pub mod vertex_buffers;
pub mod texture;

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    
    #[error("Couldn't compile shader file {path:?}: {compile_error:?}")]
    ShaderCompileError {
        compile_error: String,
        path: PathBuf,
    },
    
    #[error("Issue with shader file")]
    ShaderFileError(#[from] std::io::Error),
    
    #[error("GpuData can't be zero sized")]
    ZeroSizedGpuData,
    
    #[error("There is already a binding for this GpuData in this bindgroup")]
    GpuDataTypeAlreadyPresent,

    #[error("There doesn't exist a binding for this GpuData in this bindgroup")]
    GpuDataTypeNotPresent,
}
