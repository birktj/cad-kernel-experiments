use crate::geometry::*;
use crate::brep_1d::*;

/// A 2d boundary region.
///
/// Consists of a set of lines (sub-spaces) and a graph of edges and connections between them.
///
/// ## Invariants:
/// - Each edge has exactly two (distinct) neighbours (checked by structure)
/// - No edges intersect
/// - Each edge forms a proper 1d region
#[derive(Clone, Debug)]
pub struct Region2D {
    lines: Vec<Line>,
    edges: Vec<Edge>,
}

impl Region2D {
    pub fn new(lines: Vec<Line>, edges: Vec<Edge>) -> Option<Self> {
        let region = Self {
            lines,
            edges,
        };

        region.check()?;

        Some(region)
    }

    fn check(&self) -> Option<()> {
        for e in 0..self.edges.len() {
            let _ = self.edge_region_safe(e)?;
        }

        for e in 0..self.edges.len() {
            self.check_edge(e)?;
        }

        Some(())
    }

    // Checks if a edge is a proper 1d region.
    //
    // Also hecks that a edge is a proper node in a simple graph. Eg. siblings are also sibling of
    // this node.
    // Also checks that all other edges do not intersect with this region.
    // nodes intersects with it.
    fn check_edge(&self, e: usize) -> Option<()> {
        let edge = self.edges[e];
        let line = self.lines[edge.line];
        let r = self.edge_region(e);

        // Check that graph is valid.
        if e != self.edges[edge.x1].x1 && e != self.edges[edge.x1].x2 {
            return None
        }

        if e != self.edges[edge.x2].x1 && e != self.edges[edge.x2].x2 {
            return None
        }

        let cut_region = self.cut_region_filter(line, 
            |i| i != e && i != edge.x1 && i != edge.x2)?;

        let inside = self.inside_edge(e)?;

        if inside && !cut_region.contains(&r) {
            return None
        } else if !inside && cut_region.contains(&r) {
            return None
        }

        Some(())
    }

    fn edge_segment(&self, e: usize) -> LineSegment {
        let edge = self.edges[e];
        let line = self.lines[edge.line];
        let x1 = &self.lines[self.edges[edge.x1].line];
        let x2 = &self.lines[self.edges[edge.x2].line];
        line.line_segment(x1, x2).unwrap()
    }

    //pub fn edge_intersection(&self, e: usize, line: &Line) -> Option<>
    pub fn edge_intersects(&self, e: usize, line: &Line) -> bool {
        self.edge_segment(e).intersects_line(line)
    }

    fn inside_edge(&self, e: usize) -> Option<bool> {
        let edge = self.edges[e];
        let line = self.lines[edge.line];
        let x1 = self.lines[self.edges[edge.x1].line];
        let x2 = self.lines[self.edges[edge.x2].line];

        let mut a = line.intersection(&x1)?;
        let mut b = line.intersection(&x2)?;

        // TODO: this is stupid
        // Flip if inverted. We do this to compute a sensible region if this is an "inside" edge.
        
        Some(if a.pos < b.pos && !a.dir && b.dir {
            true
        } else if b.pos < a.pos && a.dir && !b.dir {
            true
        } else {
            false
        })
    }

    fn edge_region_safe(&self, e: usize) -> Option<Region1D> {
        let edge = self.edges[e];
        let line = self.lines[edge.line];
        let x1 = self.lines[self.edges[edge.x1].line];
        let x2 = self.lines[self.edges[edge.x2].line];

        let mut a = line.intersection(&x1)?;
        let mut b = line.intersection(&x2)?;

        // TODO: this is stupid
        // Flip if inverted. We do this to compute a sensible region if this is an "inside" edge.
        
        if a.pos < b.pos && !a.dir && b.dir {
            a.dir = true;
            b.dir = false;
        } else if b.pos < a.pos && a.dir && !b.dir {
            a.dir = false;
            b.dir = true;
        }

        Region1D::new(vec![a, b])
    }

    pub fn edge_region(&self, e: usize) -> Region1D {
        // A Region2D is only valid if all edge regions are
        self.edge_region_safe(e).unwrap()
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    fn cut_region_filter<F: FnMut(usize) -> bool>(&self, line: Line, mut f: F) -> Option<Region1D> {
        let xs = (0..self.edges().len())
            .filter(|e| f(*e))
            .filter_map(|e| {
                let e_segment = self.edge_segment(e);
                line.segment_intersection(&e_segment)
            }).collect();

        // If we cannot compute a cut region, it is proabably beacuse 
        // it is empty and we are on the edge of an edge.
        Region1D::new(xs)
    }

    pub fn cut_region(&self, line: Line) -> Region1D {
        // All cut regions of a proper Region2D should be valid
        self.cut_region_filter(line, |_| true).unwrap()
    }

    pub fn inside(&self, point: na::Point2<f64>) -> bool {
        let line = Line::from_point_dir(point, na::Vector2::x());

        let region = self.cut_region(line);

        let point_pos = line.intersection(&Line::from_point_dir(point, na::Vector2::y()))
            .unwrap().pos;

        region.inside(point_pos)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Edge {
    pub line: usize,
    pub x1: usize,
    pub x2: usize,
}

impl Edge {
    pub fn new(line: usize, x1: usize, x2: usize) -> Self {
        Self {
            line,
            x1,
            x2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lines() -> Vec<Line> {
        let p1 = na::Point2::new(0.0, 0.0);
        let p2 = na::Point2::new(3.0, 0.0);
        let p3 = na::Point2::new(3.0, 3.0);
        let p4 = na::Point2::new(0.0, 3.0);

        let p5 = na::Point2::new(1.0, 1.0);
        let p6 = na::Point2::new(2.0, 1.0);
        let p7 = na::Point2::new(2.0, 2.0);
        let p8 = na::Point2::new(1.0, 2.0);

        vec![
            Line::from_two_points(p2, p1),
            Line::from_two_points(p3, p2),
            Line::from_two_points(p4, p3),
            Line::from_two_points(p1, p4),
            Line::from_two_points(p5, p6),
            Line::from_two_points(p6, p7),
            Line::from_two_points(p7, p8),
            Line::from_two_points(p8, p5),
        ]
    }

    #[test]
    fn simple_region_valid() {
        let edges = vec![
            Edge::new(0, 1, 3),
            Edge::new(1, 0, 2),
            Edge::new(2, 1, 3),
            Edge::new(3, 0, 2),
        ];

        let region = Region2D::new(test_lines(), edges).unwrap();

        assert!(region.inside(na::Point2::new(0.5, 0.5)));
        assert!(!region.inside(na::Point2::new(3.5, 0.5)));
    }

    #[test]
    fn inverted_region_invalid() {
        let edges = vec![
            Edge::new(4, 1, 3),
            Edge::new(5, 0, 2),
            Edge::new(6, 1, 3),
            Edge::new(7, 0, 2),
        ];

        assert!(Region2D::new(test_lines(), edges).is_none());
    }

    #[test]
    fn nonintersecting_edges_region_invalid() {
        let edges = vec![
            Edge::new(4, 1, 2),
            Edge::new(5, 0, 2),
            Edge::new(6, 1, 2),
        ];

        assert!(Region2D::new(test_lines(), edges).is_none());
    }

    #[test]
    fn inconsistent_edges_region_invalid() {
        let edges = vec![
            Edge::new(0, 1, 3),
            Edge::new(1, 0, 2),
            Edge::new(2, 1, 3),
            Edge::new(7, 0, 2),
        ];

        assert!(Region2D::new(test_lines(), edges).is_none());
    }

    #[test]
    fn region_with_hole_valid() {
        let edges = vec![
            Edge::new(0, 1, 3),
            Edge::new(1, 0, 2),
            Edge::new(2, 1, 3),
            Edge::new(3, 0, 2),

            Edge::new(4, 5, 7),
            Edge::new(5, 4, 6),
            Edge::new(6, 5, 7),
            Edge::new(7, 4, 6),
        ];

        let region = Region2D::new(test_lines(), edges).unwrap();

        assert!(region.inside(na::Point2::new(0.5, 0.5)));
        assert!(!region.inside(na::Point2::new(1.5, 1.5)));
        assert!(!region.inside(na::Point2::new(3.5, 0.5)));
    }

    #[test]
    fn cut_regions_corner_stable() {
        let mut val: f64 = 10.316314;
        let mut rot: f64 = 0.0;

        for _ in 0..1000 {
            let transform = na::Rotation2::new(rot);
            let p1 = transform * na::Point2::new(3.0, 0.3);
            let p2 = transform * na::Point2::new(10.532, val);
            let p3 = transform * na::Point2::new(25.123 + val, 0.07);

            let l1 = Line::from_two_points(p1, p2);
            let l2 = Line::from_two_points(p2, p3);
            let l3 = Line::from_two_points(p3, p1);

            let region = Region2D::new(vec![l1, l2, l3], vec![
                Edge { line: 0, x1: 1, x2: 2 },
                Edge { line: 1, x1: 0, x2: 2 },
                Edge { line: 2, x1: 0, x2: 1 },
            ]).unwrap();

            // Line that touches corner but is not inside otherwise
            let ray_line_1 = Line::from_point_dir(p2, transform * na::Vector2::x());
            // Line that goes through corner via inside
            let ray_line_2 = Line::from_point_dir(p2, transform * na::Vector2::y());

            let r1 = region.cut_region(ray_line_1);
            assert!(r1.points().len() == 0 || r1.points().len() == 2);

            let r2 = region.cut_region(ray_line_2);
            assert_eq!(r2.points().len(), 2);

            val = f64::from_bits(val.to_bits()+1);
            rot = f64::from_bits(rot.to_bits()+1) + 30.0;
        }
    }
}
