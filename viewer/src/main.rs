/*
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

mod render;
mod event;
mod camera;

pub fn run(model: crate::mesh::Mesh) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(render::RenderState::new(window));

    let mut camera = camera::Trackball::new(3.14 / 4.0);
    let mut event_handler = event::EventHandler::new();

    let fill_shader = state.fill_shader().clone();
    let object = state.wireframe_object(&model)
        .hide_flat_edges()
        .build();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let mut ev = event_handler.handle_event(&event);
        camera.handle_event(&mut ev);

        state.handle_event(&event);
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => match event {
                winit::event::WindowEvent::CloseRequested
                | winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            state: winit::event::ElementState::Pressed,
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
            winit::event::Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let res = state.render(|target| {
                    fill_shader.render(target);
                    object.render(target, &camera);
                });
                match res {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    // FIXME
                    //Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            winit::event::Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
*/

extern crate nalgebra as na;
extern crate cad_kernel as kernel;

mod camera;
mod wireframe_egui;
mod canvas;

use kernel::mesh::Mesh;
use kernel::geometry;
use kernel::brep_1d::*;
use kernel::brep_2d::*;

use eframe::egui;

pub fn run(model: Mesh) {
    let mut native_options = eframe::NativeOptions::default();
    native_options.depth_buffer = 1;
    native_options.vsync = false;
    native_options.wgpu_options.present_mode = wgpu::PresentMode::Immediate;
    //native_options.wgpu_options.depth_format = Some(wgpu::TextureFormat::Depth32Float);
    eframe::run_native("CAM software", native_options, Box::new(move |cc| Box::new(App::new(cc, model))));
}

pub struct App {
    camera: camera::Trackball,
    object: std::sync::Arc<wireframe_egui::WireframeObject>,
    canvas_transform: na::Similarity2::<f64>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, model: Mesh) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        wireframe_egui::RenderContext::register(render_state);
        let renderer = render_state.renderer.read();
        let ctx = renderer.paint_callback_resources.get::<wireframe_egui::RenderContext>().unwrap();
        let object = ctx.wireframe.object(render_state, &model)
            .hide_flat_edges()
            .build();

        let camera = camera::Trackball::new(3.14 / 4.0);

        Self {
            camera,
            object: std::sync::Arc::new(object),
            canvas_transform: na::Similarity2::identity(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        wireframe_egui::RenderContext::new_frame(frame);
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("1D region");
            canvas::Canvas::new("1d_region_canvas")
                .size(egui::Vec2::new(200.0, 125.0))
                .show(ui, |canvas| {
                    let l1 = geometry::Line::from_two_points(
                        na::Point2::new(100.0, 25.0),
                        na::Point2::new(150.0, 25.0));

                    let l2 = geometry::Line::from_two_points(
                        na::Point2::new(100.0, 50.0),
                        na::Point2::new(150.0, 50.0));

                    let l3 = geometry::Line::from_two_points(
                        na::Point2::new(100.0, 75.0),
                        na::Point2::new(150.0, 75.0));


                    let l4 = geometry::Line::from_two_points(
                        na::Point2::new(100.0, 75.0),
                        na::Point2::new(150.0, 75.0));

                    let r1 = Region1D::new(vec![
                        Point1D::new(50.0, true),
                        Point1D::new(75.0, false),
                        Point1D::new(100.0, true),
                        Point1D::new(150.0, false),
                    ]).unwrap();

                    let r2 = Region1D::new(vec![
                        Point1D::new(60.0, true),
                        Point1D::new(110.0, false),
                    ]).unwrap();


                    render_1d_region(&canvas, l1, &r1);
                    render_1d_region(&canvas, l2, &r2);

                    let intersection = r1.intersection(&r2);
                    canvas.line(l3, (1.0, egui::Color32::DARK_GRAY));
                    for p in intersection.r1_inside {
                        canvas.point(l3.point(r1.points()[p].pos), egui::Color32::RED);
                    }
                    for p in intersection.r2_inside {
                        canvas.point(l3.point(r2.points()[p].pos), egui::Color32::DARK_BLUE);
                    }
                });

            ui.add_space(10.0);

            ui.heading("2D region");
            canvas::Canvas::new("2d_region_canvas")
                .size(egui::Vec2::new(300.0, 200.0))
                .show(ui, |canvas| {
                    let l1 = geometry::Line::from_two_points(
                        na::Point2::new(100.0, 50.0),
                        na::Point2::new(150.0, 150.0));

                    let l2 = geometry::Line::from_two_points(
                        na::Point2::new(130.0, 100.0),
                        na::Point2::new(150.0, 50.0));

                    let l3 = geometry::Line::from_two_points(
                        na::Point2::new(150.0, 50.0),
                        na::Point2::new(100.0, 40.0));

                    let l4 = geometry::Line::from_two_points(
                        na::Point2::new(160.0, 150.0),
                        na::Point2::new(110.0, 50.0));

                    let l5 = geometry::Line::from_two_points(
                        na::Point2::new(140.0, 50.0),
                        na::Point2::new(120.0, 100.0));

                    let l6 = geometry::Line::from_two_points(
                        na::Point2::new(100.0, 50.0),
                        na::Point2::new(150.0, 60.0));

                    /*
                    dbg!(l1.intersection(&l2));
                    dbg!(l1.intersection(&l3));
                    dbg!(l4.intersection(&l5));
                    dbg!(l4.intersection(&l6));
                    */

                    let lines = [l1, l2, l3, l4, l5, l6];

                    let region = Region2D::new(lines.into(), vec![
                        Edge { line: 0, x1: 1, x2: 2 },
                        Edge { line: 1, x1: 0, x2: 2 },
                        Edge { line: 2, x1: 1, x2: 0 },

                        Edge { line: 3, x1: 4, x2: 5 },
                        Edge { line: 4, x1: 3, x2: 5 },
                        Edge { line: 5, x1: 4, x2: 3 },
                    ]).unwrap();

                    for line in lines {
                        let proj = canvas.mouse_pos().map(|p| line.project_point(p));
                        if let Some(proj) = proj.filter(|p| p.dist.abs() <= 3.0 / canvas.scale_factor()) {
                            canvas.line(line, (1.5, egui::Color32::RED));
                        } else {
                            canvas.line(line, (1.0, egui::Color32::DARK_GRAY));
                        }
                    }

                    //canvas.line(l4, (1.0, egui::Color32::DARK_GRAY));
                    //canvas.line(l5, (1.0, egui::Color32::DARK_GRAY));
                    //canvas.line(l6, (1.0, egui::Color32::DARK_GRAY));

                    let p = l2.intersection_point(&l3).unwrap();
                    canvas.point(p, egui::Color32::DARK_GRAY);

                    //canvas.line(cut_line, (1.0, egui::Color32::DARK_BLUE));

                    if let Some(p) = canvas.mouse_pos() {
                        //let cut_line = geometry::Line::from_two_points(
                        //    na::Point2::new(100.0, 100.0),
                        //    na::Point2::new(150.0, 100.0));
                        /*
                        let cut_line = geometry::Line::from_point_dir(p, na::Vector2::x());
                        let cut_region = region.cut_region(cut_line);
                        render_1d_region(&canvas, cut_line, &cut_region);
                        */
                        if region.inside(p) {
                            canvas.point(p, egui::Color32::RED);
                        }
                    }
                });
        });
    }
}

fn render_1d_region(canvas: &canvas::CanvasUi, line: geometry::Line, region: &Region1D) {
    canvas.line(line, (1.0, egui::Color32::DARK_GRAY));

    for point in region.points() {
        canvas.point(line.point(point.pos), egui::Color32::DARK_BLUE);
    }

    for cell in region.cells() {
        let p1 = line.point(region.points()[cell[0]].pos);
        let p2 = line.point(region.points()[cell[1]].pos);
        canvas.line_segment(p1, p2, (1.5, egui::Color32::DARK_BLUE));
    }

    let proj = canvas.mouse_pos().map(|p| line.project_point(p));
    if let Some(proj) = proj.filter(|p| p.dist.abs() <= 3.0 / canvas.scale_factor()) {
        if region.inside(proj.pos) {
            canvas.point(line.point(proj.pos), egui::Color32::RED);
        } else {
            canvas.point(line.point(proj.pos), egui::Color32::DARK_GRAY);
        }
    }
}

pub fn main() {
    run(Mesh::cube(2.0));
}
