use crate::geometry;

use eframe::egui;

use std::hash::Hash;

pub struct Canvas {
    id_source: egui::Id,
    size: egui::Vec2,
    initial_mem: CanvasMemory,
    bg_color: egui::Color32,
}

impl Canvas {
    pub fn new(id_source: impl Hash) -> Self {
        Self {
            id_source: egui::Id::new(id_source),
            size: egui::Vec2::splat(100.0),
            initial_mem: CanvasMemory {
                transform: na::Similarity2::identity()
            },
            bg_color: egui::Color32::KHAKI,
        }
    }

    pub fn size(mut self, size: egui::Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn show<'a, R, F: FnOnce(CanvasUi<'a>) -> R>(self, ui: &'a mut egui::Ui, f: F) -> Option<R> {
        let id = ui.make_persistent_id(self.id_source);
        let (res, painter) = ui.allocate_painter(self.size, egui::Sense::click_and_drag());

        if ui.is_rect_visible(res.rect) {
            painter.rect_filled(res.rect, 0.0, self.bg_color);

            let mut mem = ui.memory_mut(|m| m.data.get_temp::<CanvasMemory>(id))
                .unwrap_or(self.initial_mem);

            let mut canvas = CanvasUi {
                painter,
                rect: res.rect,
                transform: mem.transform,
                res,
                ui,
            };

            if canvas.res.dragged_by(egui::PointerButton::Secondary) {
                canvas.transform.append_translation_mut(&(canvas.transform.scaling() * canvas.canvas_vec(canvas.res.drag_delta())).into());
                mem.transform = canvas.transform;
                canvas.ui.memory_mut(|m| m.data.insert_temp(id, mem.clone()));
            }

            let scroll = canvas.ui.input(|i| i.scroll_delta.y) as f64;
            if let Some(pos) = canvas.mouse_pos().filter(|_| scroll != 0.0) {
                let zoom = f64::exp(scroll * 0.01);
                canvas.transform = 
                    canvas.transform
                    * na::Translation2::from(pos.coords)
                    * na::Similarity2::from_scaling(zoom)
                    * na::Translation2::from(-pos.coords);
                mem.transform = canvas.transform;
                canvas.ui.memory_mut(|m| m.data.insert_temp(id, mem.clone()));
            }

            Some(f(canvas))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct CanvasMemory {
    transform: na::Similarity2<f64>,
}

pub struct CanvasUi<'a> {
    painter: egui::Painter,
    rect: egui::Rect,
    transform: na::Similarity2<f64>,
    res: egui::Response,
    ui: &'a mut egui::Ui,
}

impl<'a> CanvasUi<'a> {
    pub fn response(&self) -> &egui::Response {
        &self.res
    }

    pub fn ui(&'a self) -> &'a egui::Ui {
        self.ui
    }

    fn origin(&self) -> na::Point2<f64> {
        na::Point2::from(<[f32; 2]>::from(self.rect.left_bottom())).cast()
    }

    fn size(&self) -> na::Vector2<f64> {
        na::Vector2::from(<[f32; 2]>::from(self.rect.size())).cast()
    }

    pub fn gui_point(&self, p: na::Point2<f64>) -> egui::Pos2 {
        let p = self.transform * p;
        let o = self.origin();
        egui::Pos2::new((p.x + o.x) as f32, (o.y - p.y) as f32)
    }

    pub fn gui_vec(&self, v: na::Vector2<f64>) -> egui::Vec2 {
        let v = self.transform * v;
        egui::Vec2::new(v.x as f32, -v.y as f32)
    }

    pub fn canvas_point(&self, p: egui::Pos2) -> na::Point2<f64> {
        let o = self.origin();
        self.transform.inverse() * na::Point2::new(p.x as f64 - o.x, o.y - p.y as f64)
    }

    pub fn canvas_vec(&self, p: egui::Vec2) -> na::Vector2<f64> {
        self.transform.inverse() * na::Vector2::new(p.x as f64, -p.y as f64)
    }

    pub fn mouse_pos(&self) -> Option<na::Point2<f64>> {
        self.res.hover_pos().map(|p| self.canvas_point(p))
    }

    pub fn scale_factor(&self) -> f64 {
        self.transform.scaling()
    }

    fn max_point(&self) -> na::Point2<f64> {
        self.origin() + self.size()
    }

    fn edges(&self) -> [geometry::Line; 4] {
        let p0 = self.transform.inverse() * na::Point2::origin();
        let p1 = self.transform.inverse() * na::Point2::from(self.size());

        [
            geometry::Line::from_point_normal(p0, na::Vector2::x_axis()),
            geometry::Line::from_point_normal(p0, na::Vector2::y_axis()),
            geometry::Line::from_point_normal(p1, -na::Vector2::x_axis()),
            geometry::Line::from_point_normal(p1, -na::Vector2::y_axis()),
        ]
    }

    pub fn line(&self, line: geometry::Line, stroke: impl Into<egui::Stroke>) {
        let edges = self.edges();

        //dbg!(line);

        let xs = [
            line.intersection_point(&edges[0]).filter(|p| edges[1].inside(*p) && edges[3].inside(*p)),
            line.intersection_point(&edges[1]).filter(|p| edges[0].inside(*p) && edges[2].inside(*p)),
            line.intersection_point(&edges[2]).filter(|p| edges[1].inside(*p) && edges[3].inside(*p)),
            line.intersection_point(&edges[3]).filter(|p| edges[0].inside(*p) && edges[2].inside(*p)),
        ];

        //dbg!(xs);

        if let Some((sp, ep)) = xs[0].or(xs[1]).zip(xs[3].or(xs[2]).or(xs[1]))
            .or(xs[2].zip(xs[3]))
        {
            self.painter.line_segment([self.gui_point(sp), self.gui_point(ep)], stroke);
            let mid = sp + (ep - sp) / 2.0;
            self.painter.arrow(self.gui_point(mid), self.gui_vec(line.normal().into_inner() * 10.0 / self.scale_factor()), (1.0, egui::Color32::DARK_GRAY).into());
            self.painter.arrow(self.gui_point(mid), self.gui_vec(line.dir().into_inner() * 10.0 / self.scale_factor()), (1.0, egui::Color32::DARK_GRAY).into());
        }
    }

    pub fn point(&self, p: na::Point2<f64>, color: egui::Color32) {
        self.painter.circle_filled(self.gui_point(p), 2.0, color);
    }

    pub fn line_segment(&self, p1: na::Point2<f64>, p2: na::Point2<f64>, stroke: impl Into<egui::Stroke>) {
        let p1 = self.gui_point(p1);
        let p2 = self.gui_point(p2);
        self.painter.line_segment([p1, p2], stroke);
    }
}
