use eframe::egui;
use cad_kernel::geometry;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tool {
    Pointer,
    PointAdder,
}

enum DragState {
    None,
    Started(usize),
    Active(usize),
}

struct EditorLine {
    p1: usize,
    p2: usize,
}

pub struct Editor2D {
    points: Vec<na::Point2<f64>>,
    selected_points: Vec<bool>,
    dragged_point: DragState,
    lines: Vec<EditorLine>,
    selected_lines: Vec<bool>,
    tool: Tool,
}

impl Editor2D {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            selected_points: Vec::new(),
            dragged_point: DragState::None,
            lines: Vec::new(),
            selected_lines: Vec::new(),
            tool: Tool::Pointer,
        }
    }

    fn add_point(&mut self, p: na::Point2<f64>) {
        self.points.push(p);
        self.selected_points.push(false);
    }

    fn select_point(&mut self, i: usize) {
        self.selected_points[i] = !self.selected_points[i];
    }

    fn remove_point(&mut self, i: usize) {
        let mut lines_to_remove = Vec::new();
    
        for (i, l) in self.lines.iter().enumerate() {
            if self.selected_points[l.p1] || self.selected_points[l.p2] {
                lines_to_remove.push(i);
            }
        }

        for i in lines_to_remove.into_iter().rev() {
            self.lines.remove(i);
        }

        self.points.remove(i);
        self.selected_points.remove(i);
    }

    fn add_line(&mut self, l: EditorLine) {
        self.lines.push(l);
        self.selected_lines.push(false);
    }

    fn remove_line(&mut self, i: usize) {
        self.lines.remove(i);
        self.selected_lines.remove(i);
    }

    fn select_line(&mut self, i: usize) {
        self.selected_lines[i] = !self.selected_lines[i];
    }

    fn delete_selected(&mut self) {
        let selected = (0..self.points.len())
            .rev()
            .filter(|i| self.selected_points[*i])
            .collect::<Vec<_>>();

        for i in selected {
            self.remove_point(i);
        }

        let selected = (0..self.points.len())
            .rev()
            .filter(|i| self.selected_lines[*i])
            .collect::<Vec<_>>();

        for i in selected {
            self.remove_line(i);
        }
    }

    fn deselect_all(&mut self) {
        for i in 0..self.points.len() {
            self.selected_points[i] = false;
        }
        for i in 0..self.lines.len() {
            self.selected_lines[i] = false;
        }
    }

    fn selected_points<'a>(&'a self) -> impl 'a + Iterator<Item = usize> {
        self.selected_points.iter().enumerate().filter(|x| *x.1).map(|x| x.0)
    }

    fn selected_points_line(&self) -> Option<EditorLine> {
        if self.selected_points().count() == 2 {
            let p1 = self.selected_points().nth(0).unwrap();
            let p2 = self.selected_points().nth(1).unwrap();
            Some(EditorLine {
                p1,
                p2,
            })
        } else {
            None
        }
    }

    fn flip_lines(&mut self) {
        let lines = (0..self.lines.len())
            .filter(|i| self.selected_lines[*i])
            .collect::<Vec<_>>();

        for i in lines {
            let line = &mut self.lines[i];
            std::mem::swap(&mut line.p1, &mut line.p2);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.input_mut(|input| {
            if input.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete)) {
                self.delete_selected();
            }

            if input.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.deselect_all();
                self.tool = Tool::Pointer;
            }

            if input.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Enter)) {
                if let Some(l) = self.selected_points_line() {
                    self.deselect_all();
                    self.add_line(l);
                }
            }

            if input.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::F)) {
                self.flip_lines();
            }

            if input.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::A)) {
                self.deselect_all();
                self.tool = Tool::PointAdder;
            }
        });

        egui::SidePanel::left("editor_panel")
            .show_separator_line(false)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.collapsing("Points", |ui| {
                    let mut selected = None;
                    let mut removed = None;
                    for (i, p) in self.points.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                if ui.selectable_label(self.selected_points[i], format!("Point {}", i)).clicked() {
                                    selected = Some(i);
                                }
                                ui.label(egui::RichText::new(format!("({:.02}, {:.02})", p.x, p.y)).weak().small());
                                ui.add_space(5.0);
                                if ui.button("🗑").clicked() {
                                    removed = Some(i);
                                }
                            });
                        });
                    }

                    if let Some(i) = selected {
                        self.select_point(i);
                    }

                    if let Some(i) = removed {
                        self.remove_point(i);
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let size = ui.available_size();

            crate::canvas::Canvas::new("editor_canvas")
                .size(ui.available_size())
                .show(ui, |canvas| {
                    if !canvas.response().dragged_by(egui::PointerButton::Primary) {
                        self.dragged_point = DragState::None;
                    } else if let DragState::Active(i) = self.dragged_point {
                        if let Some(p) =  canvas.mouse_pos() {
                            self.points[i] = p;
                        }
                    } else if let DragState::Started(i) = self.dragged_point {
                        if let Some(p) =  canvas.mouse_pos()
                            .filter(|p| {
                                let too_far = canvas.ui().input(|i| i.pointer.press_origin())
                                    .map(|o| o.distance(canvas.gui_point(*p)) > 6.0).unwrap_or(false);
                                let too_long = canvas.ui().input(|i| i.pointer.press_start_time()
                                    .map(|t| i.time - t > 0.6)).unwrap_or(false);

                                too_far || too_long
                            })
                        {
                            self.dragged_point = DragState::Active(i);
                            self.points[i] = p;
                        }
                    }

                    if let Some(l) = self.selected_points_line() {
                        let line = geometry::Line::from_two_points(self.points[l.p1], self.points[l.p2]);

                        canvas.line(line, (1.0, egui::Color32::GRAY));
                    }

                    let mut hover_point = None;
                    let mut selected = None;
                    for (i, p) in self.points.iter().enumerate() {
                        if let Some(_) = canvas.mouse_pos()
                            .filter(|mp| (mp - p).norm() < 5.0 / canvas.scale_factor())
                            .filter(|_| self.tool == Tool::Pointer)
                        {
                            hover_point = Some(i);
                            canvas.point(*p, egui::Color32::RED);
                            if canvas.response().clicked() {
                                selected = Some(i);
                            } else if canvas.response().drag_started_by(egui::PointerButton::Primary) {
                                self.dragged_point = DragState::Started(i);
                            }
                        } else if self.selected_points[i] {
                            canvas.point(*p, egui::Color32::BLUE);
                        } else {
                            canvas.point(*p, egui::Color32::DARK_GRAY);
                        }
                    }

                    if let Some(i) = selected {
                        self.select_point(i);
                    } else if let Some(p) = canvas.mouse_pos()
                        .filter(|_| self.tool == Tool::PointAdder) 
                    {
                        if canvas.response().clicked() {
                            self.add_point(p);
                        }

                        let p1 = p - canvas.canvas_vec(egui::Vec2::new(20.0, 0.0));
                        let p2 = p + canvas.canvas_vec(egui::Vec2::new(20.0, 0.0));
                        canvas.line_segment(p1, p2, (1.0, egui::Color32::DARK_GRAY));

                        let p1 = p - canvas.canvas_vec(egui::Vec2::new(0.0, 20.0));
                        let p2 = p + canvas.canvas_vec(egui::Vec2::new(0.0, 20.0));
                        canvas.line_segment(p1, p2, (1.0, egui::Color32::DARK_GRAY));
                    }

                    let mut selected = None;
                    for (i, l) in self.lines.iter().enumerate() {
                        let line = geometry::Line::from_two_points(self.points[l.p1], self.points[l.p2]);

                        if let Some(proj) = canvas.mouse_pos()
                            .filter(|_| hover_point.is_none())
                            .map(|p| line.project_point(p))
                            .filter(|proj| proj.dist.abs() < 3.0 / canvas.scale_factor())
                        {
                            canvas.line(line, (1.5, egui::Color32::RED));
                            if canvas.response().clicked() {
                                selected = Some(i);
                            }
                        } else if self.selected_lines[i] {
                            canvas.line(line, (1.0, egui::Color32::BLUE));
                        } else {
                            canvas.line(line, (1.0, egui::Color32::DARK_GRAY));
                        }
                    }

                    if let Some(i) = selected {
                        self.select_line(i);
                    }
                });
        });

    }
}
