use winit::{
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use smol_renderer::*;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
fn create_depth_texture(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::Texture {
    let desc = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device.create_texture(&desc)
}

#[repr(C)]
struct VertexData {
    pos: [f32; 3],
}

unsafe impl GpuData for VertexData {
    fn as_raw_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.pos.as_ptr() as *const u8,
                self.pos.len() * std::mem::size_of::<f32>(),
            )
        }
    }
}

impl VertexBufferData for VertexData {
    fn get_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<VertexData> as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float3],
        }
    }
}

// create new Vertexbuffer struct which acts as a vec where you can add VertebufferData and get a buffer?
fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let left_corner = VertexData {
        pos: [-1.0, 0.0, 0.0],
    };
    let right_corner = VertexData {
        pos: [1.0, 0.0, 0.0],
    };
    let top = VertexData {
        pos: [0.0, 1.0, 0.0],
    };
    // maybe incorrect order
    let bytes = vec![right_corner, top, left_corner];
    let bytes = bytes
        .iter()
        .map(GpuData::as_raw_bytes)
        .flatten()
        .copied()
        .collect::<Vec<u8>>();
    device.create_buffer_with_data(bytes.as_slice(), wgpu::BufferUsage::VERTEX)
}

fn create_node<'a>(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> RenderNode<'a> {
    let vertex_shader = VertexShader::builder()
        .set_shader_file("examples/basic/shader.vs")
        .build(device)
        .unwrap();
    let fragment_shader = FragmentShader::builder()
        .set_shader_file("examples/basic/shader.fs")
        .build(device)
        .unwrap();

    RenderNode::builder()
        .add_vertex_buffer::<VertexData>()
        .set_vertex_shader(vertex_shader)
        .set_fragment_shader(fragment_shader)
        .build(device, color_format, DEPTH_FORMAT)
        .unwrap()
}


async fn run_example(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let surface = wgpu::Surface::create(&window);
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        },
        wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: Default::default(),
        })
        .await;

    let swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    let depth_texture = create_depth_texture(&device, &swap_chain_desc);
    let depth_texture_view = depth_texture.create_default_view();
    let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
    //let format = swap_chain_desc.format;
    let render_node = create_node(&device, swap_chain_desc.format);
    let buffer = create_vertex_buffer(&device);

    event_loop.run(move |event, _, control_flow| {
        let _ = &buffer;
        match event {
            event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {} 
            },
            event::Event::RedrawRequested(_) => {
                let frame = swap_chain.get_next_texture().unwrap();
                let mut encoder = 
                    device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                render_node.run(
                    &mut encoder,
                    &wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Load,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            },
                        }],
                        depth_stencil_attachment: Some(
                            wgpu::RenderPassDepthStencilAttachmentDescriptor {
                                attachment: &depth_texture_view,
                                depth_load_op: wgpu::LoadOp::Clear,
                                depth_store_op: wgpu::StoreOp::Store,
                                clear_depth: 1.0,
                                stencil_load_op: wgpu::LoadOp::Clear,
                                stencil_store_op: wgpu::StoreOp::Store,
                                clear_stencil: 0,
                            },
                        ),
                    },
                    |node, pass| {
                        pass.set_vertex_buffer(0, &buffer, 0, 0);
                    },
                )
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Example")
        .build(&event_loop)
        .unwrap();

    futures::executor::block_on(run_example(event_loop, window));
}
