use crate::RenderError;
use shaderc::{Compiler, CompileOptions, ShaderKind};
use std::fs::File;
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use wgpu::ShaderModuleSource;

fn compile_glsl<'a>(path: impl AsRef<Path>, shader_type: ShaderKind) -> Result<&'a [u32], RenderError> {
    let mut file = File::open(&path)?;
    let mut src = String::new();
    let mut compiler = Compiler::new().expect("Can't create shader compiler");
    let mut options = CompileOptions::new().expect("Can't create compiler options");
    file.read_to_string(&mut src)?;
    let spirv = compiler
        .compile_into_spirv(&src, shader_type, "test.glsl", "main", Some(&options))
        .map_err(|err| RenderError::ShaderCompileError {
            compile_error: err.to_string(),
            path: PathBuf::from(path.as_ref()),
        })?;
    Ok(spirv.as_binary())
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
        let data = compile_glsl(path, ShaderKind::Vertex)?;
        let module = device.create_shader_module(ShaderModuleSource::SpirV(&data));
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
        let data = compile_glsl(path.as_ref(), ShaderKind::Fragment)?;
        let module = device.create_shader_module(ShaderModuleSource::SpirV(&data));
        Ok(FragmentShader { module })
    }

    pub(crate) fn get_descriptor(&self) -> wgpu::ProgrammableStageDescriptor {
        get_descriptor(&self.module)
    }
}
