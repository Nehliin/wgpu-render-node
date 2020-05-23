use crate::RenderError;
use glsl_to_spirv::ShaderType;
use std::fs::File;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

fn compile_glsl(path: impl AsRef<Path>, shader_type: ShaderType) -> Result<Vec<u32>, RenderError> {
    let mut file = File::open(&path)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;
    let spirv = glsl_to_spirv::compile(&src, shader_type).map_err(|err| {
        RenderError::ShaderCompileError {
            compile_error: err,
            path: PathBuf::from(path.as_ref()),
        }
    })?;

    let data = wgpu::read_spirv(spirv)?;
    Ok(data)
}

#[inline(always)]
fn get_descriptor(module: &wgpu::ShaderModule) -> wgpu::ProgrammableStageDescriptor {
    wgpu::ProgrammableStageDescriptor {
        module,
        entry_point: "main",
    }
}

pub struct VertexShader {
    module: wgpu::ShaderModule,
}

impl VertexShader {
    pub fn new(device: &wgpu::Device, path: impl AsRef<Path>) -> Result<VertexShader, RenderError> {
        let data = compile_glsl(path, ShaderType::Vertex)?;
        let module = device.create_shader_module(&data);
        Ok(VertexShader { module })
    }

    pub(crate) fn get_descriptor(&self) -> wgpu::ProgrammableStageDescriptor {
        get_descriptor(&self.module)
    }
}

pub struct FragmentShader {
    module: wgpu::ShaderModule,
}

impl FragmentShader {
    pub fn new(
        device: &wgpu::Device,
        path: impl AsRef<Path>,
    ) -> Result<FragmentShader, RenderError> {
        let data = compile_glsl(path.as_ref(), ShaderType::Fragment)?;
        let module = device.create_shader_module(&data);
        Ok(FragmentShader { module })
    }

    pub(crate) fn get_descriptor(&self) -> wgpu::ProgrammableStageDescriptor {
        get_descriptor(&self.module)
    }
}
