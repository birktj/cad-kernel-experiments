#[derive(Copy, Clone)]
struct Line {
    n: na::UnitVector2<f64>,
    d: f64,
}

impl Line {
    pub fn from_point_normal(p0: na::Point2<f64>, n: na::UnitVector2<f64>) -> Self {
        let d = -n.into_inner().dot(&p0.coords);
        Self {
            n,
            d
        }
    }

    pub fn dir_vector(&self) -> na::Vector<f64> {
        na::Vector2::new(self.n.y, self.n.x)
    }

    pub fn point(&self, p: f64) -> na::Point2<f64> {
        na::Point2::origin() 
            + self.n.into_inner() * self.d
            + self.dir_vector() * p
    }

    pub fn intersection(&self, other: &Line) -> Option<Point1D> {
        let det = self.n.perp(&other.n);
        if det.abs() < EPSILON {
            // Might be colinear, but still not a valid intersection
            None
        } else {
            let u = other.n.into_inner() * other.d  - self.n.into_inner() * self.d;
            let a = u.perp(&(-other.dir_vector())) / det;
            //let b = self.dir_vector().perp(&u) / det;
            Some([a, b])
        }
    }

    pub fn intersection_point(&self, other: &Line) -> Option<na::Point2<f64>> {
        Some(self.point(self.intersection(other)?.pos))
    }
}

pub struct Region {
    lines: Vec<Line>,
    edges: Vec<Edge>,
}

impl Region {
    pub fn new(lines: Vec<Line>, edges: Vec<Edge>) -> Option<Self> {
        let region = Self {
            lines,
            edges,
        };
        // TODO: check if region is valid
        Some(region)
    }

    fn edge_region(&self, e: usize) -> Option<[f64; 2]> {
        let line = self.lines[self.edges[e].line];
        let start_line = self.lines[self.edges[edge.start].line];
        let end_line = self.lines[self.edges[edge.end].line];

        let a = line.intersection(&start_line)?[0];
        let b = line.intersection(&end_line)?[0];

        Some([a.min(b), a.max(b)])
    }

    // Checks if a edge properly intersects with start and end and also no other
    // nodes intersects with it. Also check that the inside of start and end is the same as
    // this edge.
    fn check_edge(&self, e: usize) -> bool {
        (|| {
            let edge = self.edges[e];
            let [a, b] = self.edge_region(e)?;

            for (i, e2) in self.edges.iter().enumerate() {
                if i == e || i == edge.start || i == edge.end {
                    continue
                }

                if let Some([x, y]) = line.intersection(&self.lines[e2.line]) {
                    if let Some([a2, b2]) = self.edge_region(i) {
                        if x > a && x < b && y > a2 && y < b2 {
                            return None
                        }
                    }
                }
            }
        })().is_some()
    }
}

pub struct Edge {
    line: usize,
    start: usize,
    end: usize,
}

pub struct Point1D {
    pos: f64,
    dir: f64,
}

pub struct Region1D {
    a: Point1D,
    b: Point1D,
}

impl Region1D {
    pub fn new(a: Point1D, b: Point1D) -> Option<Self> {
    }
}
