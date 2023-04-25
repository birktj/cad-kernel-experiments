pub const EPSILON: f64 = 1e-5;

#[derive(Copy, Clone, Debug)]
pub struct Line {
    n: na::UnitVector2<f64>,
    d: f64,
}

impl Line {
    pub fn from_two_points(p0: na::Point2<f64>, p1: na::Point2<f64>) -> Self {
        Self::from_point_dir(p1, p1 - p0)
    }

    pub fn from_point_dir(p0: na::Point2<f64>, d: na::Vector2<f64>) -> Self {
        let n = na::Unit::new_normalize(na::Vector2::new(d.y, -d.x));
        Self::from_point_normal(p0, n)
    }

    pub fn from_point_normal(p0: na::Point2<f64>, n: na::UnitVector2<f64>) -> Self {
        let d = n.into_inner().dot(&p0.coords);
        Self {
            n,
            d
        }
    }

    pub fn with_origin(&self, p: na::Point2<f64>) -> Self {
        let d = self.n.into_inner().dot(&p.coords) + self.d;
        Self {
            n: self.n,
            d
        }
    }

    pub fn through_point(&self, p: na::Point2<f64>) -> Self {
        Self::from_point_normal(p, self.n)
    }

    pub fn origin(&self) -> na::Point2<f64> {
        na::Point2::from(self.normal().into_inner() * self.d)
    }

    pub fn dir(&self) -> na::UnitVector2<f64> {
        na::Unit::new_normalize(na::Vector2::new(-self.n.y, self.n.x))
    }

    pub fn normal(&self) -> na::UnitVector2<f64> {
        self.n
    }

    pub fn point(&self, p: f64) -> na::Point2<f64> {
        na::Point2::origin() 
            + self.n.into_inner() * self.d
            + self.dir().into_inner() * p
    }

    pub fn intersection(&self, other: &Line) -> Option<LinePoint> {
        let det = self.n.perp(&other.n);
        // TODO: can we throw away the epsilon here?
        if det.abs() < EPSILON {
            // Might be colinear, but still not a valid intersection
            None
        } else {
            let u = other.n.into_inner() * other.d  - self.n.into_inner() * self.d;
            let pos = u.perp(&other.dir().into_inner()) / det;
            //let b = self.dir().perp(&u) / det;
            let dir = det.signum() > 0.0;
            Some(LinePoint {
                pos,
                dir,
            })
        }
    }

    /// Computes the intersection point of this line with another.
    ///
    /// This function computes the intersection point in such a way that calling
    /// `intersection_point` on `other` with `self` as an argument always gives the same result as
    /// calling it on `self` with `other` as an argument.
    pub fn intersection_point(&self, other: &Line) -> Option<na::Point2<f64>> {
        let p1 = self.point(self.intersection(other)?.pos);
        let p2 = other.point(other.intersection(self)?.pos);

        if total_cmp_vec2(p1.coords, p2.coords).is_lt() {
            Some(p1)
        } else {
            Some(p2)
        }
    }

    pub fn project_point(&self, point: na::Point2<f64>) -> PointProjection {
        let l1 = Line::from_point_normal(point, self.normal());
        let l2 = Line::from_point_normal(point, self.dir());
        let x1 = l2.intersection(&l1).unwrap();
        let x2 = l2.intersection(&self).unwrap();
        let x3 = self.intersection(&l2).unwrap();

        PointProjection {
            pos: x3.pos,
            dist: (x1.pos - x2.pos) * x2.dirsign(),
            inside: x1.pos * x2.dirsign() >= x2.pos * x2.dirsign(),
        }
    }

    pub fn inside(&self, point: na::Point2<f64>) -> bool {
        self.project_point(point).inside
    }

    /// A line segment og this line given by the intersection of two other lines.
    ///
    /// This function computes the endpoints in a stable way such the the points
    /// are the exact same if called on `x1` or `x2` with this line as one of
    /// the arguments.
    pub fn line_segment<'a>(&self, mut x1: &'a Line, mut x2: &'a Line) -> Option<LineSegment> {
        if self.intersection(x1)?.pos > self.intersection(x2)?.pos {
            std::mem::swap(&mut x1, &mut x2);
        }
        Some(LineSegment {
            p1: self.intersection_point(x1)?,
            p2: self.intersection_point(x2)?,
        })
    }

    /// Computes the intersection of this line with a line segment.
    ///
    /// In contrast to [`Self::intersection`] this is stable such that given two segments that
    /// share a point this will compute a usefull intersection for both:
    /// - If this line goes through the shared corner of the two segmements one and only one
    ///   segment will intersect.
    /// - If this line touches the shared corner (but does not go through) then either none or both
    ///   of the segments will inteersect and the position of the intersections will respect the 
    ///   orderof the segments.
    pub fn segment_intersection(&self, segment: &LineSegment) -> Option<LinePoint> {
        let mut proj1 = self.project_point(segment.p1);
        let mut proj2 = self.project_point(segment.p2);
        let mut dir = false;

        if proj2.dist < proj1.dist {
            std::mem::swap(&mut proj1, &mut proj2);
            dir = true;
        }

        //dbg!(proj1, proj2);

        if proj1.dist < 0.0 && proj2.dist >= 0.0 {
            // TODO: should we use an epsilon here?
            //let pos = proj1.pos + (proj2.pos - proj1.pos) * (1.0 -  proj2.dist / (proj2.dist - proj1.dist));
            let pos = proj2.pos - (proj2.pos - proj1.pos) * proj2.dist / (proj2.dist - proj1.dist);
            Some(LinePoint {
                pos,
                dir,
            })
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LineSegment {
    p1: na::Point2<f64>,
    p2: na::Point2<f64>,
}

impl LineSegment {
    // This is not public as 
    fn line(&self) -> Line {
        Line::from_two_points(self.p1, self.p2)
    }

    pub fn intersects_line(&self, line: &Line) -> bool {
        line.segment_intersection(&self).is_some()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LinePoint {
    pub pos: f64,
    pub dir: bool,
}

impl LinePoint {
    pub fn new(pos: f64, dir: bool) -> Self {
        Self {
            pos,
            dir,
        }
    }

    pub fn dirsign(&self) -> f64 {
        if self.dir {
            1.0
        } else {
            -1.0
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PointProjection {
    pub pos: f64,
    pub dist: f64,
    pub inside: bool,
}

/// Computes a determenistic and stable ordering of two vectors.
///
/// This function is not supposed to compute a sensible ordering, but rather one that is both
/// numericaly stable and deterministic.
fn total_cmp_vec2(v1: na::Vector2<f64>, v2: na::Vector2<f64>) -> std::cmp::Ordering {
    v1.x.total_cmp(&v2.x).then(v1.y.total_cmp(&v2.y))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_intersection_and_segment_intersection_same() {
        let p1 = na::Point2::new(3.0, 7.0);
        let p2 = na::Point2::new(2.5, -10.0);

        let line = Line::from_two_points(p1, p2);

        let segment = line.line_segment(
            &Line::from_point_dir(p1, na::Vector2::x()),
            &Line::from_point_dir(p2, na::Vector2::x())).unwrap();

        let ray_line = Line::from_point_dir(na::Point2::origin(), na::Vector2::x());

        let x1 = ray_line.intersection(&line).unwrap();
        let x2 = ray_line.segment_intersection(&segment).unwrap();

        assert_eq!(x1.dir, x2.dir);
        assert!((x1.pos - x2.pos).abs() < EPSILON, "{:?} and {:?}", x1.pos, x2.pos);
    }

    #[test]
    fn line_intersection_point_stable() {
        let mut val: f64 = -1.316314;
        let mut misses = 0;

        for _ in 0..1000 {
            val = f64::from_bits(val.to_bits()+1);

            let p1 = na::Point2::new(3.0, 10.0);
            let p2 = na::Point2::new(1.543, val);
            let p3 = na::Point2::new(5.123, -10.0);

            let l1 = Line::from_two_points(p1, p2);
            let l2 = Line::from_two_points(p2, p3);

            let x1 = l1.intersection_point(&l2).unwrap();
            let x2 = l2.intersection_point(&l1).unwrap();

            assert_eq!(x1.x, x2.x);
            assert_eq!(x1.y, x2.y);
        }
    }

    #[test]
    fn line_segment_intersection_stable() {
        let mut val: f64 = -1.316314;
        let mut rot: f64 = 0.0;

        let mut misses = 0;

        for _ in 0..10000 {
            let transform = na::Rotation2::new(rot);
            let p1 = transform * na::Point2::new(3.0, 10.0);
            let p2 = transform * na::Point2::new(1.543, val);
            let p3 = transform * na::Point2::new(5.123, -10.0);

            let l1 = Line::from_two_points(p1, p2);
            let l2 = Line::from_two_points(p2, p3);

            let ray_line = Line::from_point_dir(p2, transform * na::Vector2::x());

            let l1_e = l1.intersection(&l2).unwrap().pos;
            let l2_s = l2.intersection(&l1).unwrap().pos;

            let l1_x = l1.intersection(&ray_line).unwrap().pos;
            let l2_x = l2.intersection(&ray_line).unwrap().pos;

            // Check that the cut line slips through
            if !(l1_x <= l1_e || l2_x >= l2_s) {
                misses += 1;
            }

            let p1_cut = Line::from_point_dir(p1, transform * na::Vector2::x());
            let p3_cut = Line::from_point_dir(p3, transform * na::Vector2::x());

            let l1_segment = l1.line_segment(&p1_cut, &l2).unwrap();
            let l2_segment = l2.line_segment(&l1, &p3_cut).unwrap();

            let x1 = l1_segment.intersects_line(&ray_line);
            let x2 = l2_segment.intersects_line(&ray_line);

            // Check that at least one segment intersects
            assert!(x1 || x2);
            // Check that at most one segment intersects
            assert!(x1 != x2);

            val = f64::from_bits(val.to_bits()+1);
            rot = f64::from_bits(rot.to_bits()+1) + 30.0;
        }

        assert!(misses > 100);
    }

    #[test]
    fn line_segment_intersection_corner() {
        let mut val: f64 = 10.316314;
        let mut rot: f64 = 0.0;

        let mut hits = 0;

        for _ in 0..1000 {
            let transform = na::Rotation2::new(rot);
            let p1 = transform * na::Point2::new(3.0, 0.3);
            let p2 = transform * na::Point2::new(10.532, val);
            let p3 = transform * na::Point2::new(25.123, 0.07);

            let l1 = Line::from_two_points(p1, p2);
            let l2 = Line::from_two_points(p2, p3);
            let l3 = Line::from_two_points(p3, p1);

            let l1_seg = l1.line_segment(&l2, &l3).unwrap();
            let l2_seg = l2.line_segment(&l3, &l1).unwrap();

            let ray_line = Line::from_point_dir(p2, transform * na::Vector2::x());

            let x1 = ray_line.segment_intersection(&l1_seg);
            let x2 = ray_line.segment_intersection(&l2_seg);

            // Either both intersect or none do
            assert!(x1.is_some() == x2.is_some());

            if let Some((x1, x2)) = x1.zip(x2) {
                hits += 1;

                // Check that the intersection pos of l1 is always before that of l2
                assert!(x1.pos <= x2.pos);
            }
            val = f64::from_bits(val.to_bits()+1);
            rot = f64::from_bits(rot.to_bits()+1) + 30.0;
        }

        // Check that we at least get some cases where both intersect, but not all.
        assert!(hits > 50 && hits < 500);
    }
}
