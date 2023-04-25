//use super::event::{Event, EventMsg};
use eframe::egui;

#[derive(Copy, Clone)]
pub struct Trackball {
    fov: f32,
    pitch: f32,
    yaw: f32,
    dist: f32,
    target: na::Point3<f32>,
    window_width: f32,
    window_height: f32,
}

impl Trackball {
    pub fn new(fov: f32) -> Trackball {
        Trackball {
            fov,
            pitch: 1.0,
            yaw: 2.0,
            dist: 25.0,
            target: na::Point3::new(0.,0.,0.),
            window_width: 1.0,
            window_height: 1.0,
        }
    }

    /*
    pub fn handle_event(&mut self, event: &mut Event) {
        match event.msg {
            EventMsg::MouseMove(dx, dy) => {
                if event.state.left_press {
                    self.rotate([dx, dy]);
                }
                if event.state.right_press {
                    self.pan([dx, dy]);
                }
            },
            EventMsg::Scroll(d) => self.zoom(d),
            _ => (),
        }
    }
    */

    pub fn egui_sense() -> egui::Sense {
        egui::Sense::drag()
    }

    pub fn handle_egui_response(&mut self, res: &egui::Response) {
        if res.dragged_by(egui::PointerButton::Primary) {
            self.rotate(res.drag_delta().into());
        }
        if res.hovered() {
            let zoom = res.ctx.input(|i| i.scroll_delta.y/3.0);
            self.zoom(zoom);
        }
    }

    pub fn update_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }

    pub fn rotate(&mut self, dd: [f32; 2]) {
        let dx = dd[0] * 0.01;
        let dy = dd[1] * 0.01;

        self.pitch -= dy;
        self.pitch = f32::max(self.pitch, 0.01);
        self.pitch = f32::min(self.pitch, std::f32::consts::PI-0.01);
        self.yaw -= dx;
    }

    pub fn zoom(&mut self, d: f32) {
        self.dist *= f32::exp(-d * 0.1);
        self.dist = f32::max(self.dist, 1.0);
    }

    pub fn pan(&mut self, dd: [f32; 2]) {
        let dir       = (self.target - self.eye()).normalize();
        let tangent   = na::Vector3::z().cross(&dir).normalize();
        let bitangent = dir.cross(&tangent);
        let mult      = self.dist / 1000.0;
        
        self.target = self.target
            + tangent * (dd[0] * mult) 
            + bitangent * (dd[1] * mult);
    }

    pub fn target(&self) -> na::Point3<f32> {
        self.target
    }

    pub fn look_at(&mut self, target: na::Point3<f32>) {
        self.target = target;
    }

    pub fn view(&self) -> na::Isometry3<f32> {
        let rot =
            na::UnitQuaternion::new(na::Vector3::x() * std::f32::consts::PI / -2.0)
            * na::UnitQuaternion::new(na::Vector3::z() * std::f32::consts::PI / 2.0);
        
        na::Isometry3::look_at_rh(&(rot * self.eye()), &(rot * self.target), &na::Vector3::y())
            * rot
    }

    pub fn projection(&self) -> na::Projective3<f32> {
        let aspect = self.window_width / self.window_height;
        na::Perspective3::new(aspect, self.fov, 0.5, 250.0)
            .to_projective()
    }

    pub fn unproject_point(&self, point: na::Point2<f32>) -> (na::Point3<f32>, na::Vector3<f32>) {
        let norm_x = 2.0 * point.coords.x / self.window_width - 1.0;
        let norm_y = 2.0 * -point.coords.y / self.window_height + 1.0;
        let p1 = na::Point3::new(norm_x, norm_y, -1.0);

        let p2 = na::Point3::new(norm_x, norm_y, 1.0);

        let p1 = self.transformation().inverse_transform_point(&p1);
        let p2 = self.transformation().inverse_transform_point(&p2);

        (p1, (p2 - p1).normalize())
    }

    pub fn eye(&self) -> na::Point3<f32> {
        let px = self.target.x + self.dist * self.yaw.cos() * self.pitch.sin();
        let py = self.target.y + self.dist * self.yaw.sin() * self.pitch.sin();
        let pz = self.target.z + self.dist * self.pitch.cos();

        na::Point3::new(px, py, pz)
    }

    pub fn transformation(&self) -> na::Projective3<f32> {
        self.projection() * self.view()
    }
}
