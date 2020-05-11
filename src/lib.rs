use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

pub mod uniforms;

type Device = ();

// CONST GENERICS?

trait Raw {}
struct VertexBufferData<T: Raw> {
    data: PhantomData<T>,
    buffer: wgpu::Buffer,
}
// impl Deref till buffer
impl<T: Raw> VertexBufferData<T> {
    fn gpu_allocate(data: &T) -> Self {
        unimplemented!();
    }
}

struct VertexBuffer<T: Raw> {
    marker: PhantomData<T>,
    step_mode: wgpu::InputStepMode,
    attributes: Vec<wgpu::VertexAttributeDescriptor>,
    //  descriptor: wgpu::VertexBufferDescriptor<'a>,
}

impl<T: Raw> VertexBuffer<T> {
    fn new() -> Self {
        let step_mode = wgpu::InputStepMode::Vertex;
        let attributes = vec![wgpu::VertexAttributeDescriptor {
            offset: 0,
            format: wgpu::VertexFormat::Float3,
            shader_location: 0,
        }];

        Self {
            marker: PhantomData::default(),
            step_mode,
            attributes,
            //  descriptor,
        }
    }

    fn get_layout<'a>(&'a self) -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<T>() as u64,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }

    fn set_data(&self, data: &VertexBufferData<T>) {}
}

struct Uniform<T: Raw> {
    marker: PhantomData<T>,
    //buffer: wgpu::Buffer,
    //bind_group: wgpu::BindGroup,
    //bind_group_layout: wgpu::BindGroupLayout,
}

impl<T: Raw> Uniform<T> {
    fn new(device: &Device) -> Self {
        Self {
            marker: PhantomData::default(),
        }
    }
    fn update(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        unimplemented!();
    }
}

struct VertexShader<T: Raw, E: Raw> {
    vertex_buffer: VertexBuffer<T>,
    shader: String, // temp
    uniform: Option<Uniform<E>>,
}

impl<T: Raw + Default, E: Raw + Default> VertexShader<T, E> {
    fn builder(device: &Device, shader: String) -> Self {
        VertexShader {
            vertex_buffer: VertexBuffer::new(),
            shader: String::new(),
            uniform: None,
        }
    }

    fn set_vertex_buffer(mut self, vertex_buffer: VertexBuffer<T>) -> Self {
        self.vertex_buffer = vertex_buffer;
        self
    }

    fn set_uniform(mut self, uniform: Uniform<E>) -> Self {
        self.uniform = Some(uniform);
        self
    }

    fn get_runner(&mut self) -> VertexShaderRunner<T, E> {
        VertexShaderRunner {
            vertex_shader: self,
        }
    }
}

struct VertexShaderRunner<'a, T: Raw, E: Raw> {
    vertex_shader: &'a mut VertexShader<T, E>,
}

impl<'a, T: Raw, E: Raw> VertexShaderRunner<'a, T, E> {
    fn set_vertex_data(&mut self, data: &VertexBufferData<T>) {
        self.vertex_shader.vertex_buffer.set_data(data);
    }

    fn upload_uniform(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        data: E,
    ) {
        //self.vertex_shader.uniform.unwrap().buffer = staging buffer
    }
}

struct Node<T: Raw, E: Raw> {
    vertex_shader: Option<VertexShader<T, E>>,
}

impl<'a, T: Raw + Default, E: Raw + Default> Node<T, E> {
    pub fn new() -> Self {
        Self {
            vertex_shader: None,
        }
    }

    pub fn set_vertex_shader(mut self, vertex_shader: VertexShader<T, E>) -> Self {
        self.vertex_shader = Some(vertex_shader);
        self
    }

    fn get_vertex_runner(&'a mut self) -> VertexShaderRunner<'a, T, E> {
        self.vertex_shader.as_mut().unwrap().get_runner()
    }
}

// testa runt node lite

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct MeshVertex;
    impl Raw for MeshVertex {}
    #[derive(Default)]
    struct UniformData;

    impl Raw for UniformData {}
    #[test]
    fn it_works() {
        let vertex_data = MeshVertex;
        let device = &();
        let vertex_buffer: VertexBuffer<MeshVertex> = VertexBuffer::new();
        let uniform_data: Uniform<UniformData> = Uniform::new(device);

        let mut node = Node::new().set_vertex_shader(
            VertexShader::builder(device, String::from("hej"))
                .set_vertex_buffer(vertex_buffer)
                .set_uniform(uniform_data),
        );

        let test = node.get_vertex_runner();

        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_construction() {
        /* let device = &();
        let global_uniform: Uniform<GlobalUniform> = Uniform::new(device);
        let local_uniform: Uniform<LocalUniform> = Uniform::new(device);
        let Graph::builder()
                .set_global_uniform(global_uniform)
                .add_node(Node {
                    vertex_shader: VertexShader::new("somefile.glsl"),

                })*/
    }
}
