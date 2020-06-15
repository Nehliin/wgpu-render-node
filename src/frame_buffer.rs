
pub struct ReadableDepthStencilFramebuffer {
    format: wgpu::TextureFormat,
    texture: wgpu::Texture,
}
// make clear defaults are set and make it more customizable?
impl ReadableDepthStencilFramebuffer {
    pub fn new(device: &wgpu::Device, desc: wgpu::DepthStencilStateDescriptor, size: wgpu::Extent3d) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ReadableDepthStencilFrameBuffer"),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: desc.format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        ReadableDepthStencilFramebuffer {
            format: desc.format,
            texture,
        }
    }



//    pub fn create_readable_texture() -> 

}
// this can create default depth texture if non readable
// from swapchainDescriptor?
/*struct DepthStencilBuilder {
    size: CustomSizeType, // include size + dimension + array_layer_count
    format: format,
    compare_func,
    stencil_front,
    stencil_back,
    read/write mask
   sensible defaults here 


   1.  abstract? texture, loadable texture traits
    2. simple texture implements loadable texture (or struct specific method)
    3. DepthStencilBuffer implements abstract texture, texture data needs to be changed so it can handle multiple views?
    4. DepthStencil buffer DATA is attached to rendernode as depthstencil target, only a single texture view is used
    5. depth stencil buffer can be used as a texture cuz of trait imple
    how to handle swapchain texture view?

}*/


trait FrameBufferGen<T> {
    fn create_readable(builder: InfoObj) -> ReadableFrameBuffer<T>;
    fn create(info: InfoObj) -> FrameBuffer<T>   
}


trait FrameBuffer {
    const DESC: wgpu::DepthStencilStateDescriptor;

    fn new(device: &wgpu::Device, layer: usize, size: wgpu::Extent3d) -> Self;


}

/*
Trait ReadableDepthStencilFrambuffer
trait DepthStencilFramebuffer

it's always a single texture

1. model pass needs a single bindgroup to read the texture
2. shadow pass needs different views to the different layers in the framebuffer


RenderNode .add_texture(ReadableFrameBuffer) -> create bindgroup and attach to pipeline
RenderNode .set_depth_stencil_target<ReadableFrameBuffer>()

RenderNodePass .set_depth_target(FrameBufferTarget) 
**/
