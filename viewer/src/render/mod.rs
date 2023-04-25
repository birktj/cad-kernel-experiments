use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

mod wireframe;
mod fill;

pub use wireframe::{WireframeObject, WireframeObjectBuilder};
pub use fill::FillShader;

pub struct RenderContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    framebuffer: wgpu::TextureView,
    framebuffer_desc: wgpu::TextureDescriptor<'static>,
    depth: wgpu::TextureView,
    depth_desc: wgpu::TextureDescriptor<'static>,
    size: winit::dpi::PhysicalSize<u32>,
    samples: u32,
}

pub struct RenderTarget<'c, 'v> {
    ctx: &'c RenderContext,
    encoder: wgpu::CommandEncoder,
    view: &'v wgpu::TextureView,
    resolve_target: Option<&'v wgpu::TextureView>,
}

pub struct RenderState {
    surface: wgpu::Surface, // must be dropped before window
    ctx: RenderContext,
    window: Window,
    renderer_wireframe: wireframe::Wireframe,
    renderer_fill: FillShader,
}

impl RenderState {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let samples = 4;
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        // surface is also dropped before window as it is decleared before window.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
    
        surface.configure(&device, &config);

        let framebuffer_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };

        let framebuffer = device.create_texture(&framebuffer_desc)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };

        let depth = device.create_texture(&depth_desc)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let ctx = RenderContext {
            device,
            queue,
            config,
            framebuffer,
            framebuffer_desc,
            depth,
            depth_desc,
            size,
            samples,
        };

        let renderer_wireframe = wireframe::Wireframe::new(&ctx);
        let renderer_fill      = FillShader::new(&ctx);

        Self {
            surface,
            ctx,
            window,
            renderer_wireframe,
            renderer_fill,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.ctx.size = new_size;
            self.ctx.config.width = new_size.width;
            self.ctx.config.height = new_size.height;
            self.surface.configure(&self.ctx.device, &self.ctx.config);
            self.ctx.framebuffer_desc.size.width = new_size.width;
            self.ctx.framebuffer_desc.size.height = new_size.height;
            self.ctx.framebuffer = self.ctx.device.create_texture(&self.ctx.framebuffer_desc)
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.ctx.depth_desc.size.width = new_size.width;
            self.ctx.depth_desc.size.height = new_size.height;
            self.ctx.depth = self.ctx.device.create_texture(&self.ctx.depth_desc)
                .create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>) {
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == &self.window().id() => match event {
                WindowEvent::Resized(physical_size) => {
                    self.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.resize(**new_inner_size);
                }
                _ => {}
            }
            _ => {}
        }
    }

    pub fn render<F>(&mut self, f: F) -> Result<(), wgpu::SurfaceError> 
        where for<'t, 'c, 'v> F: FnOnce(&'t mut RenderTarget<'c, 'v>)
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut target = RenderTarget {
            ctx: &self.ctx,
            encoder,
            view: &self.ctx.framebuffer,
            resolve_target: Some(&view),
        };

        f(&mut target);

        self.ctx.queue.submit(std::iter::once(target.encoder.finish()));
        output.present();

        Ok(())
    }
}

impl RenderState {
    pub fn wireframe_object<'a>(&'a self, geometry: &'a crate::mesh::Mesh) -> WireframeObjectBuilder<'a, impl FnMut([usize; 2]) -> bool> {
        self.renderer_wireframe.object(&self.ctx, geometry)
    }

    pub fn fill_shader(&self) -> &FillShader {
        &self.renderer_fill
    }
}
