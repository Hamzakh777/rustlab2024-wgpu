// The tasks for this chapter are split into the different methods of Application.
// Go through the methods from top to bottom.
// Once all your methods are fully implemented, start your application and make sure
// it displays two white triangles.
// You can of course already try running your application inbetween to ensure no
// validation errors are raised.
// Afterwards, continue with adjusting your shaders in `application.wgsl`.
//
// Refer to https://docs.rs/wgpu/latest/wgpu/ to learn about a type's constructor,
// methods and attributes.
use std::{borrow::Cow, sync::Arc};
use anyhow::{Context, Result};
use wgpu::{core::device, InstanceDescriptor, Limits, MemoryHints, PipelineLayout, PipelineLayoutDescriptor, RenderPassDescriptor, RenderPipeline, ShaderModuleDescriptor, TextureViewDescriptor};
use winit::{dpi::PhysicalSize, window::Window};

pub struct Application {
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: RenderPipeline,
}

impl Application {
    pub async fn new(window: Arc<Window>, size: PhysicalSize<u32>) -> Result<Self> {
        // 1. We first must create a `wgpu::Instance`.
        // This is the entrypoint to all communication with wgpu.
        let instance = wgpu::Instance::new(InstanceDescriptor::default());

        // 2. Next, we create our surface through the instance we created above.
        // For this, we must pass a window for the surface to target.
        // A surface is what anything we draw will be displayed on.
        // the surface is the link to the windown
        let surface = instance.create_surface(window).expect("Creating a window failed.");

        // 3. Once we have our surface, we request an adapter that is compatible with
        // this surface from our wgpu instance.
        // We want to request a high performance GPU so in case our device is a laptop
        // with two GPUs, we get the more powerful one.
        // Note that reqeusting an adapter is an asynchronous operation that must be awaited.
        // If no adapter matches our request options, we receive `None`.
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await.expect("Failed to find an appropriate adapter");

        // 4. While an adapter represents the a physical GPU, we also need a logical handle
        // to this GPU that enforces feature and memory limitations and is responsible for
        // executing any GPU commands we feed it.
        // This logical handle is called a "device" and can be requested from the adapter
        // we created above.
        // As we have no special requirements at this moment we just request the default
        // features and limits.
        // Requesting a device from an adapter returns a tuple containing both the device
        // and a queue to which we can submit GPU commands.
        // Note that requesting a device again is an asynchronous operation.
        let (device, queue)= adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            memory_hints: MemoryHints::MemoryUsage,
            required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
        }, None).await.expect("Failed to create a device");

        // 5. Get the default config for our adapter from the surface, using the size
        // we got as parameter to our constructor. Make sure the size has a width and
        // height of at least 1, otherwise creating the surface may fail.
        // This only returns None if the surface and adapter are incompatible.
        // As we requested the adapter with `compatible_surface`, this is never the case.
        let surface_config = surface.get_default_config(&adapter, size.width, size.height).unwrap();

        // 6. Configure the surface using our logical device and the surface config.
        surface.configure(&device, &surface_config);

        // 7. Load the shader source code from `application.wgsl` and create a shader module
        // on our logical device to which we pass the loaded code as source.
        // As shader source type, we use WGSL.
        // You can optionally pass a label that will be used when reporting errors regarding
        // this particular shader module.
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("application.wgsl")))
        });

        // 8. Define the layout for our pipeline by creating a pipeline layout on our device.
        // Our layout is very basic for now, so it is sufficient to use the PipelineLayoutDescriptor's
        // default initializer.
        // This is where we define how it should draw things, the format of the vertex, and what should
        // happen at every step of the pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            // all the resources
            bind_group_layouts: &[],
            label: None,
            push_constant_ranges: &[]
        });

        // 9. Next, create the render pipeline itself on the device.
        // This requires:
        // - layout: Our pipeline layout created above.
        // - vertex: A description of our pipeline's VertexState. This receives our shader module
        //   and optionally the name of the entry_point function inside that shader module
        //   As we only have one vertex shader in our code, this can be set to None for
        //   automatic detection.
        //   We don't need any buffer and no special compilation options.
        // - fragment: A description of our pipeline's FragmentState. This receives our shader module
        //   and optionally the name of the entry_point function inside that shader module
        //   As we only have one fragment shader in our code, this can be set to None for
        //   automatic detection.
        //   Also, we must define the color targets inside our fragment state.
        //   We only have one color target, which is defined by our surface_config's format,
        //   and should use a replacement blend (overwriting colors of the previous render)
        //   as well as write all color components our shaders return.
        //   We don't need any special compilation options.
        // - primitive: A description of our pipeline's PrimitiveState. This defines what
        //   kind of geometric primitive will be used in our render pipeline.
        //   We use the default primitive, a triangle list.
        // All other parameters may use their defaults.
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Save these for later use
        Ok(Self {
            surface_config,
            surface,
            device,
            queue,
            render_pipeline,
        })
    }

    pub fn resize(&mut self, window: &Window, width: u32, height: u32) {
        log::info!("Resize: {}x{}", width, height);

        // 1. Update our surface_config to the new dimensions.
        // Note that in rare scenarios, we may receive a width or height
        // of zero. Ensure the configured surface has a width and height
        // of at least one, otherwise we will run into validation issues.
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);

        // 2. Reconfigure our surface using the updated surface_config
        self.surface.configure(&self.device, &self.surface_config);

        // On macos the window needs to be redrawn manually after resizing
        window.request_redraw();
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        winit_event: &winit::event::WindowEvent,
    ) -> bool {
        false
    }

    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        // Relevant wgpu types for this method:
        // - SurfaceTexture, Texture, TextureView (not in presentation)
        // - CommandEncoder, CommandEncoderDescriptor
        // - RenderPass, RenderPassDescriptor
        // - RenderPassColorAttachment, Operations, LoadOp, StoreOp, Color

        // 1. To render something to the screen, we must first request the current
        // texture from our surface.
        let frame = self.surface.get_current_texture().expect("Failed to get current texture");

        // 2. A texture itself cannot be used as render target.
        // We must create a view from this texture that then contains the metadata
        // our render pipeline needs to render to it.
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        // 3. All commands to be enqueued to our GPU's queue must first be encoded
        // so they are compatible with our logical device.
        // For this, we create a command encoder using our device.
        let mut encoder= self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // 4. Defining rendering commands for a GPU happens in form of a render pass.
        // We create a render pass by "beginning" it on the command encoder.
        // To actually get something out of the render pass, we give it a slice of
        // color attachments to render to (in our case, just one).
        // This color attachment receives the view we created for our surface texture earlier.
        // We then tell it what operations (ops) to perform on this view:
        // - On load, clear the surface texture using a black color
        // - On store, overwrite the contents of the surface texture (simply called "Store")

        // 5. To let the render pass know of the structure of our pipeline, such as
        // shaders, or geometric primitives, set its pipeline to the render pipeline
        // we created in our constructor.

        // 6. Tell the render pass to draw six vertices (must be passed as a range 0 to 6)
        // for one instance (again, as a range 0 to 1).
        // Instancing will not be covered in this workshop.

        // 7. Before finishing our command encoder, we must drop the
        // render pass so it knows it is complete.

        {
            // you don't need the `render pass` once you submit it
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..6, 0..1);

        }
        
        // 8. Finish the command encoder, returning a command buffer.
        // Then, submit the command buffer to our GPU queue.
        self.queue.submit(Some(encoder.finish()));
        
        // 9. Present the frame (our SurfaceTexture)
        frame.present();

        Ok(())
    }
}
