mod utils;

use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, Vector3};
use smol_renderer::*;
use utils::*;
use winit::{
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
#[derive(Clone)]
pub struct ModelInfo {
    pub isometry: Isometry3<f32>,
}

#[derive(Clone)]
pub struct Camera {
    direction: Vector3<f32>,
    position: Point3<f32>,
    view_matrix: Matrix4<f32>,
    projection_matrix: Perspective3<f32>,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        direction: Vector3<f32>,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let view_target = position + direction;
        Camera {
            direction,
            position,
            view_matrix: Matrix4::look_at_rh(&position, &view_target, &Vector3::new(0.0, 1.0, 0.0)),
            projection_matrix: Perspective3::new(
                window_width as f32 / window_height as f32,
                45.0,
                0.1,
                100.0,
            ),
        }
    }
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

    let (cube, command_buffer) = create_cube(&device);
    queue.submit(&[command_buffer]);

    let camera = Camera::new(
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -1.0),
        size.width,
        size.height,
    );
    let mut model_info = ModelInfo {
        isometry: Isometry3::translation(0.0, 0.0, -7.0),
    };
    model_info
        .isometry
        .append_rotation_wrt_center_mut(&nalgebra::UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            0.3,
        ));
    let vertex_shader = VertexShader::new(&device, "examples/basic/shader.vs").unwrap();
    let fragment_shader = FragmentShader::new(&device, "examples/basic/shader.fs").unwrap();

    let render_node = RenderNode::builder()
        .add_vertex_buffer::<Vertex>()
        .set_vertex_shader(vertex_shader)
        .set_fragment_shader(fragment_shader)
        .add_uniform_bind_group(
            UniformBindGroup::builder()
                .add_binding::<CameraGpuData>(wgpu::ShaderStage::VERTEX)
                .unwrap()
                .add_binding::<RawModelInfo>(wgpu::ShaderStage::VERTEX)
                .unwrap()
                .build(&device),
        )
        .add_texture::<SimpleTexture>(wgpu::ShaderStage::FRAGMENT)
        .build(&device, swap_chain_desc.format, DEPTH_FORMAT)
        .unwrap();
    event_loop.run(move |event, _, control_flow| {
        let _ = (
            &render_node,
            &cube,
            &camera,
            &device,
            &queue,
            &swap_chain,
            &swap_chain_desc,
        );
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
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                render_node
                    .update(
                        &device,
                        &mut encoder,
                        0,
                        &CameraGpuData::from(camera.clone()),
                    )
                    .unwrap();
                render_node
                    .update(
                        &device,
                        &mut encoder,
                        0,
                        &RawModelInfo::from(model_info.clone()),
                    )
                    .unwrap();
                let mut runner = render_node.runner(
                    &mut encoder,
                    wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.1,
                                g: 0.7,
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
                );
                runner.set_vertex_buffer_data(0, &cube.vertices);
                runner.set_index_buffer(&cube.index_buf, 0, 0);
                runner.set_texture_data(1, &cube.texture);
                runner.draw_indexed(0..cube.index_count, 0, 0..1);
                drop(runner);
                queue.submit(&[encoder.finish()]);
            }
            _ => {}
        }
    });
}
// TODO: SEGFAULT IF TEXTURE BINDGROUP ISNT ADDED TO PIPELINE??
fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Example")
        .build(&event_loop)
        .unwrap();

    futures::executor::block_on(run_example(event_loop, window));
}
/*
render node runnder which accepts a vec of vertexbuffer data and texture data
expects a list of drawable, that trait includes types of VertexBufferData and TextureData
both are shit when a texture isn't connected to a vertex buffer eg shadow map

render node runner which have equivalent methods as render pass but with typed methods,
it keeps track of correct indexes as well
self.set_vertex_buffer etc

*/
