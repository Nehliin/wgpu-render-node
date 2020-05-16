use crate::RenderError;
use glsl_to_spirv::ShaderType;
use std::fs::File;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

fn compile_glsl(path: PathBuf, shader_type: ShaderType) -> Result<Vec<u32>, RenderError> {
    let mut file = File::open(&path)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;
    let spirv = glsl_to_spirv::compile(&src, shader_type).map_err(|err| {
        RenderError::ShaderCompileError {
            compile_error: err,
            path,
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
    pub fn builder() -> VertexShaderBuilder {
        VertexShaderBuilder {
            path: PathBuf::new(),
        }
    }

    pub fn get_descriptor(&self) -> wgpu::ProgrammableStageDescriptor {
        get_descriptor(&self.module)
    }
}
pub struct VertexShaderBuilder {
    path: PathBuf,
}

impl VertexShaderBuilder {
    pub fn set_shader_file(mut self, path: impl AsRef<Path>) -> Self {
        self.path = PathBuf::from(path.as_ref());
        self
    }

    pub fn build(self, device: &wgpu::Device) -> Result<VertexShader, RenderError> {
        let data = compile_glsl(self.path, ShaderType::Vertex)?;
        let module = device.create_shader_module(&data);
        Ok(VertexShader { module })
    }
}

impl FragmentShader {
    pub fn builder() -> FragmentShaderBuilder {
        FragmentShaderBuilder {
            path: PathBuf::new(),
        }
    }

    pub fn get_descriptor(&self) -> wgpu::ProgrammableStageDescriptor {
        get_descriptor(&self.module)
    }
}
pub struct FragmentShader {
    module: wgpu::ShaderModule,
}

pub struct FragmentShaderBuilder {
    path: PathBuf,
}

impl FragmentShaderBuilder {
    pub fn set_shader_file(mut self, path: impl AsRef<Path>) -> Self {
        self.path = PathBuf::from(path.as_ref());
        self
    }

    pub fn build(self, device: &wgpu::Device) -> Result<FragmentShader, RenderError> {
        let data = compile_glsl(self.path, ShaderType::Fragment)?;
        let module = device.create_shader_module(&data);
        Ok(FragmentShader { module })
    }
}
