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

    pub fn intersection(&self, other: &Line) -> Option<[f64; 2]> {
        let det = self.n.perp(&other.n);
        if det.abs() < EPSILON {
            // Might be colinear, but still not a valid intersection
            None
        } else {
            let u = other.n.into_inner() * other.d  - self.n.into_inner() * self.d;
            let a = u.perp(&(-other.dir_vector())) / det;
            let b = self.dir_vector().perp(&u) / det;
            Some([a, b])
        }
    }
}

pub type PolygonRef = usize;

pub struct PolygonSplit {
    polygon: PolygonRef,
    new_inside: Vec<Polygon>,
    new_outside: Vec<Polygon>,
}

pub struct RegionIntersection2D {
    splits: Vec<PolygonSplit>,
    inside_other: Vec<PolygonRef>,
    new_lines: Vec<Line>,
}

/// 2d region
pub struct Region2D {
    cells: Vec<Polygon>,
    lines: Vec<Line>, // surfaces
}

impl Region2D {
    pub fn intersection(&self, other: &Region2D) -> RegionIntersection2D {
        let line_intersection = |l1, l2| self.lines[l1].intersection(&other.lines[l2]);
        // Algorithm:
        // keep track of what lines from the other region we need to add.
        // Maybe check if any are the same?
        //
        //
        // Then we compute which polygons in this region is inside the other
        // for each polygon A in this region:
        //      count number of polygons in the other region that contain A 
        //      if odd then add A as inside the other
        //
        // Now we can go through the polygons that intersect
        // for each polygon A in this region:
        //      keep track of what to split A into
        //      for each polygon B in the other region:
        //          if A and B intersect
    //              or alternativly if any of the splits of A intersect with B:
        //              add the intersection lines from B
        //              compute the split of A (should be doable only in the abstract?)
    }

    // This is actually a cell in 1d in the space of the line
    fn edge_segment(&self, edge: PolygonEdge) -> Cell1D {
        let i1 = self.lines[edge.line].intersection(&self.lines[edge.start_intersection])
            .unwrap();
        let i2 = self.lines[edge.line].intersection(&self.lines[edge.end_intersection])
            .unwrap();

        [i1[0], i2[0]]
    }
}

/// Cell in 2d
pub struct Polygon {
    // A polygon is represented as a list of lines where each edge of the polygon continues
    // until the intersection of the next line.
    // Self-intersections are not allowed so an important invariant is that 
    lines: Vec<usize>,
}

impl Polygon {
    pub fn edges<'a>(&'a self) -> impl 'a + Iterator<Item = PolygonEdge> {
        self.lines.iter()
            .zip(self.lines.iter().cycle().skip(1))
            .zip(self.lines.iter().cycle().skip(2))
            .map(|((l1, l2), l3)| PolygonEdge {
                line: l2,
                start_intersection: l1,
                end_intersection: l3,
            })
    }
}

pub struct PolygonRef<'a> {
    region: &'a Region2D,
    polygon: &'a Polygon,
}

impl<'a> PolygonRef<'a> {
}

#[derive(Copy, Clone)]
pub struct IntersectionRef2D {
    l1: usize,
    l2: usize,
}

#[derive(Copy, Clone)]
pub struct PolygonEdge {
    line: usize,
    start_intersection: usize,
    end_intersection: usize,
}


impl IntersectionRef2D {
    pub fn connected(&self, other: &Self) -> bool {
        self.l1 == other.l1 || self.l1 == other.l2 || self.l2 == other.l1 || self.l2 == other.l2
    }
}

pub struct Region1D {
    cells: Vec<[usize; 2]>,
    points: Vec<f64>,
}

// We don't care about 1d regions, only 1d cells
pub struct Cell1D {
    start: f64,
    end: f64,
}

impl Cell1D {
    pub fn inside(&self, other: &Cell1D) -> bool {
        self.start >= other.start && self.end <= other.end
    }

    pub fn intersection(&self, other: &Cell1D) -> Option<Cell1D> {
        if self.start < other.start && self.end > other.start && self.end < other.end {
            Some(Cell1D {
                start: other.start,
                end:  self.end,
            })
        } else if self.start > other.start && self.start < other.end && self.end > other.end {
            Some(Cell1D {
                start: other.start,
                end:  self.end,
            })
        } else {
            None.
        }
    }
}
