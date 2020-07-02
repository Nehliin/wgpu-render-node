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
    let instace = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instace.create_surface(&window);
        (size, surface)
    };

    let unsafe_features = wgpu::UnsafeFeatures::disallow();

    let adapter = instace
        .request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            unsafe_features,
        )
        .await
        .unwrap();
    let features = adapter.features();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features,
                shader_validation: true,
                limits: Default::default(),
            },
            None,
        )
        .await
        .unwrap();

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

    let cube = create_cube(&device, &queue);

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
        .add_local_uniform_bind_group(
            UniformBindGroup::builder()
                .add_binding::<CameraGpuData>(wgpu::ShaderStage::VERTEX)
                .unwrap()
                .add_binding::<RawModelInfo>(wgpu::ShaderStage::VERTEX)
                .unwrap()
                .build(&device),
        )
        .add_texture::<SimpleTexture>()
        .add_default_color_state_desc(swap_chain_desc.format)
        .set_default_rasterization_state()
        .set_default_depth_stencil_state()
        .build(&device)
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
                let frame = swap_chain.get_next_frame().unwrap().output;
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
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.7,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: Some(
                            wgpu::RenderPassDepthStencilAttachmentDescriptor {
                                attachment: &depth_texture_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: true,
                                }),
                                stencil_ops: None,
                            },
                        ),
                    },
                );
                runner.set_vertex_buffer_data(0, &cube.vertices);
                runner.set_index_buffer(cube.index_buf.slice(..));
                runner.set_texture_data(0, &cube.texture);
                runner.draw_indexed(0..cube.index_count, 0, 0..1);
                drop(runner);
                queue.submit(vec![encoder.finish()]);
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
