use super::{RenderContext, RenderTarget};
use crate::mesh::Mesh;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use std::collections::HashMap;

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

#[derive(Clone)]
pub struct Wireframe {
    inner: Arc<Inner>,
}

struct Inner {
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    uniforms_bind_group: wgpu::BindGroup,
}

impl Wireframe {
    pub fn new(ctx: &RenderContext) -> Self {
        let uniforms = ctx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("wireframe uniform buffer"),
                contents: &[0; std::mem::size_of::<Uniforms>()],
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let uniform_bind_group_layout = 
            ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        
        let uniforms_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                }
            ],
            label: Some("wireframe uniform bind group"),
        });

        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/wireframe.wgsl"));

        let pipeline_layout =
            ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: ctx.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
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
        pub fn object<'a>(&self, ctx: &'a RenderContext, geometry: &'a Mesh) -> WireframeObjectBuilder<'a, impl FnMut([usize; 2]) -> bool> {
        WireframeObjectBuilder {
            renderer: self.clone(),
            ctx,
            geometry,
            mark_edge_f: |_edge| { true },
        }
    }
}

pub struct WireframeObject {
    vertices: wgpu::Buffer,
    vertex_count: u32,
    renderer: Wireframe,
}

impl WireframeObject {
    pub fn render(&self, target: &mut RenderTarget, camera: &crate::viewer::camera::Trackball) {
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

        target.ctx.queue.write_buffer(&self.renderer.inner.uniforms, 0, bytemuck::cast_slice(&[uniforms]));
        
        let mut render_pass = target.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wireframe render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.view,
                resolve_target: target.resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.ctx.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.renderer.inner.pipeline);
        render_pass.set_bind_group(0, &self.renderer.inner.uniforms_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}

pub struct WireframeObjectBuilder<'a, F> {
    renderer: Wireframe,
    geometry: &'a Mesh,
    ctx: &'a RenderContext,
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
            vertices,
            vertex_count: shape.len() as u32,
            renderer: self.renderer,
        }
    }
}
