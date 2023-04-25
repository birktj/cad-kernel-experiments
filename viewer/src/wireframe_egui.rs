use kernel::mesh::Mesh;
use wgpu::util::DeviceExt;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use eframe::egui_wgpu;
use eframe::egui::epaint;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    //color: [f32; 3],
    barycentric: [f32; 3],
    normal: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    matrix: [[f32; 4]; 4],
    //view_matrix: [[f32; 3]; 3],
    //__align: [u8; 12],
    wireframe_color: [f32; 4],
    face_color_light: [f32; 4],
    face_color_dark: [f32; 4],
    light_dir: [f32; 3],
    __align: [u8; 4],
}

pub struct RenderContext {
    pub frame: FrameStorage,
    pub wireframe: Wireframe,
}

impl RenderContext {
    pub fn register(state: &egui_wgpu::RenderState) {
        let ctx = RenderContext {
            frame: FrameStorage {
                objects: Vec::new(),
            },
            wireframe: Wireframe::register(state),
        };
        
        state.renderer.write()
            .paint_callback_resources.insert(ctx);
    }

    pub fn new_frame(frame: &eframe::Frame) {
        let mut renderer = frame.wgpu_render_state().unwrap()
            .renderer.write();
        let mut ctx = renderer.paint_callback_resources.get_mut::<Self>()
            .unwrap();
        ctx.frame.objects = Vec::new();
    }
}

#[derive(Clone)]
pub struct Wireframe {
    inner: Arc<Inner>,
}

pub struct FrameStorage {
    objects: Vec<Arc<dyn std::any::Any + Send + Sync>>,
}

pub struct FrameStorageHandle<T> {
    i: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for FrameStorageHandle<T> {
    fn clone(&self) -> Self {
        Self {
            i: self.i,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Copy for FrameStorageHandle<T> {}

impl<T> FrameStorageHandle<T> {
    pub fn empty() -> Self {
        Self {
            i: usize::MAX,
            _marker: std::marker::PhantomData,
        }
    }
}

impl FrameStorage {
    pub fn store<T: 'static + Send + Sync>(&mut self, val: Arc<T>) -> FrameStorageHandle<T> {
        self.objects.push(val);

        FrameStorageHandle {
            i: self.objects.len()-1,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get<T: std::any::Any>(&self, handle: FrameStorageHandle<T>) -> Option<&T> {
        self.objects.get(handle.i).and_then(|a| a.downcast_ref())
    }
}

struct Inner {
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    uniforms_bind_group: wgpu::BindGroup,
}

impl Wireframe {
    pub fn register(state: &egui_wgpu::RenderState) -> Self {
        let uniforms = state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("wireframe uniform buffer"),
                contents: &[0; std::mem::size_of::<Uniforms>()],
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let uniform_bind_group_layout = 
            state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("wireframe uniform bind group layout"),
            });
        
        let uniforms_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                }
            ],
            label: Some("wireframe uniform bind group"),
        });

        let shader = state.device.create_shader_module(wgpu::include_wgsl!("render/shaders/wireframe.wgsl"));

        let pipeline_layout =
            state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wireframe render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, //Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            //depth_stencil: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            /*{
                count: ctx.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            */
            multiview: None,
        });

        Self {
            inner: Arc::new(Inner {
                pipeline,
                uniforms,
                uniforms_bind_group,
            })
        }
    }

    pub fn object<'a>(&self, ctx: &'a egui_wgpu::RenderState, geometry: &'a Mesh) -> WireframeObjectBuilder<'a, impl FnMut([usize; 2]) -> bool> {
        WireframeObjectBuilder {
            renderer: self.clone(),
            ctx,
            geometry,
            mark_edge_f: |_edge| { true },
        }
    }
}

pub struct WireframeObject {
    vertices: Arc<wgpu::Buffer>,
    vertex_count: u32,
    renderer: Wireframe,
    frame_store_idx: Mutex<FrameStorageHandle<wgpu::Buffer>>,
}

impl WireframeObject {
    pub fn prepare(&self, ctx: &mut RenderContext, device: &wgpu::Device, queue: &wgpu::Queue, camera: &crate::camera::Trackball) {
        let light_dir = camera.view().inverse() * na::Vector3::new(6.0, 6.0, 10.0);
        let x_axis = camera.view() * na::Vector::x();
        let y_axis = camera.view() * na::Vector::y();
        let z_axis = camera.view() * na::Vector::z();
        let uniforms = Uniforms {
            matrix: (*camera.transformation().matrix()).into(),
            //view_matrix: (*camera.view().rotation.to_rotation_matrix().matrix()).into(),
            //__align: [0; 12],
            wireframe_color: [0.0, 0.0, 0.0, 1.0],
            face_color_light: [0.7,0.1,0.1, 1.0],
            face_color_dark: [0.7 * 0.3,0.1 * 0.3,0.1 * 0.3, 1.0],
            light_dir: light_dir.into(),
            __align: [0; 4],
        };

        queue.write_buffer(&self.renderer.inner.uniforms, 0, bytemuck::cast_slice(&[uniforms]));
        *self.frame_store_idx.lock().unwrap() = ctx.frame.store(self.vertices.clone());
    }

    pub fn paint<'a, 'c>(&'a self, ctx: &'c RenderContext, render_pass: &'a mut wgpu::RenderPass<'c>) {
        render_pass.set_pipeline(&ctx.wireframe.inner.pipeline);
        render_pass.set_bind_group(0, &ctx.wireframe.inner.uniforms_bind_group, &[]);
        render_pass.set_vertex_buffer(0, ctx.frame.get(*self.frame_store_idx.lock().unwrap()).unwrap().slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}

pub struct WireframeObjectBuilder<'a, F> {
    renderer: Wireframe,
    geometry: &'a Mesh,
    ctx: &'a egui_wgpu::RenderState,
    mark_edge_f: F,
}

impl<'a, F: 'a + FnMut([usize; 2]) -> bool> WireframeObjectBuilder<'a, F> {
    pub fn mark_edges<F2: FnMut([usize; 2]) -> bool>(self, f: F2) -> WireframeObjectBuilder<'a, F2> {
        WireframeObjectBuilder {
            renderer: self.renderer,
            ctx: self.ctx,
            geometry: self.geometry,
            mark_edge_f: f
        }
    }

    pub fn hide_flat_edges(self) -> WireframeObjectBuilder<'a, impl 'a + FnMut([usize; 2]) -> bool> {
        let mut trimap = HashMap::new();

        for f in &self.geometry.faces {
            let mut ab = [f[0], f[1]];
            ab.sort();
            let mut ac = [f[0], f[2]];
            ac.sort();
            let mut bc = [f[1], f[2]];
            bc.sort();

            trimap.entry(ab).or_insert(Vec::new())
                .push(f[2]);
            trimap.entry(ac).or_insert(Vec::new())
                .push(f[1]);
            trimap.entry(bc).or_insert(Vec::new())
                .push(f[0]);
        }

        let geometry = self.geometry;

        self.mark_edges(move |[a, b]| {
            let mut iter = trimap.get(&[a, b]).into_iter().flatten();
            let a = geometry.vertices[a];
            let b = geometry.vertices[b];

            iter.next().zip(iter.next())
                .map(|(c, d)| {
                    let c = geometry.vertices[*c];
                    let d = geometry.vertices[*d];
                    let normal = (b - a)
                        .cross(&(c - a))
                        .normalize();

                    (d - a).normalize().dot(&normal).abs() > 1e-4
                }).unwrap_or(true)
        })
    }

    pub fn build(mut self) -> WireframeObject {
        let mut shape: Vec<Vertex> = Vec::new();

        for f in &self.geometry.faces {
            let a = self.geometry.vertices[f[0]];
            let b = self.geometry.vertices[f[1]];
            let c = self.geometry.vertices[f[2]];
            let normal = (b - a)
                .cross(&(c - a))
                .normalize();

            let mut ab = [f[0], f[1]];
            ab.sort();
            let mut ac = [f[0], f[2]];
            ac.sort();
            let mut bc = [f[1], f[2]];
            bc.sort();

            let c_bary = if (self.mark_edge_f)(ab) { 0.0 } else { 1.0 };
            let b_bary = if (self.mark_edge_f)(ac) { 0.0 } else { 1.0 };
            let a_bary = if (self.mark_edge_f)(bc) { 0.0 } else { 1.0 };

            shape.extend(
                [
                    Vertex {
                        position: a.coords.into(),
                        barycentric: [1.0, b_bary, c_bary],
                        normal: normal.into(),
                    },
                    Vertex {
                        position: b.coords.into(),
                        barycentric: [a_bary, 1.0, c_bary],
                        normal: normal.into(),
                    },
                    Vertex {
                        position: c.coords.into(),
                        barycentric: [a_bary, b_bary, 1.0],
                        normal: normal.into(),
                    },
                ].iter(),
            );
        }

        let vertices = self.ctx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("wireframe vertex buffer"),
                contents: bytemuck::cast_slice(&shape),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        WireframeObject {
            vertices: Arc::new(vertices),
            vertex_count: shape.len() as u32,
            renderer: self.renderer,
            frame_store_idx: Mutex::new(FrameStorageHandle::empty()),
        }
    }
}
