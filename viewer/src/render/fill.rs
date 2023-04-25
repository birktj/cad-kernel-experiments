use super::{RenderContext, RenderTarget};
use wgpu::util::DeviceExt;
use std::sync::Arc;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    color: [f32; 4]
}

#[derive(Clone)]
pub struct FillShader {
    inner: Arc<Inner>,
}

struct Inner {
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    uniforms_bind_group: wgpu::BindGroup,
}

impl FillShader {
    pub(super) fn new(ctx: &RenderContext) -> Self {
        let uniforms = ctx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("fill uniform buffer"),
                contents: &[0; std::mem::size_of::<Uniforms>()],
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let uniform_bind_group_layout = 
            ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("fill uniform bind group layout"),
            });
        
        let uniforms_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                }
            ],
            label: Some("fill uniform bind group"),
        });

        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/fill.wgsl"));

        let pipeline_layout =
            ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fill render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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

    pub fn render(&self, target: &mut RenderTarget) {
        let uniforms = Uniforms {
            color: [0.05, 0.05, 0.05, 1.0],
        };

        target.ctx.queue.write_buffer(&self.inner.uniforms, 0, bytemuck::cast_slice(&[uniforms]));
        
        let mut render_pass = target.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wireframe render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.view,
                resolve_target: target.resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
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

        render_pass.set_pipeline(&self.inner.pipeline);
        render_pass.set_bind_group(0, &self.inner.uniforms_bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
