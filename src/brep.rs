const EPSILON: f64 = 1e-5;

#[derive(Copy, Clone)]
struct Plane {
    n: na::UnitVector3<f64>,
    d: f64,
}

impl Plane {
    pub fn from_point_normal(p0: na::Point3<f64>, n: na::UnitVector3<f64>) -> Self {
        let d = -n.into_inner().dot(&p0.coords);
        Self {
            n,
            d
        }
    }

    pub fn from_point_two_vectors(p0: na::Point3<f64>, u: na::Vector3<f64>, w: na::Vector3<f64>) -> Option<Self> {
        let normal = na::Unit::try_new(u.cross(&w), EPSILON)?;
        Some(Self::from_point_normal(p0, normal))
    }

    pub fn from_three_points(p0: na::Point3<f64>, p1: na::Point3<f64>, p2: na::Point3<f64>) -> Option<Self> {
        Self::from_point_two_vectors(p0, p1 - p0, p2 - p0)
    }

    pub fn normal(&self) -> na::UnitVector3<f64> {
        self.n
    }
}

pub struct Region3 {
    points: Vec<na::Point3<f64>>,
    faces: Vec<Shell>,
}

pub struct Shell {
    faces: Vec<Face3>,
}

pub struct Face3 {
    loops: Vec<Loop>,
    plane: Plane,
}

pub struct Region2 {
    points: Vec<na::Point2<f64>>,
    loops: Vec<Loop>, 
}

pub struct Loop {
    segments: Vec<[usize; 2]>,
}

pub struct Region1 {
    points: Vec<f64>,
    segments: Vec<(usize, usize)>,
}

pub struct Segment2 {
    a: na::Point2<f64>,
    b: na::Point2<f64>,
}

pub struct Ray2 {
    s: na::Point2<f64>,
    d: na::Vector2<f64>,
}

impl Segment2 {
    pub fn ray_intersection(&self, ray: &Ray2) -> Option<na::Point2<f64>> {
        // https://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect/565282#565282
        let d = self.b - self.a;
        if ray.d.perp(&d) < EPSILON && (self.a - ray.s).perp(&d) < EPSILON {
            // Colinear

        } else if ray.d.perp(&d) < EPSILON {
            // Parallel
            None
        } else {
        }
    }
}

impl Loop {
    pub fn edges<'a>(&'a self) -> impl 'a + Iterator<Item = [usize; 2]> {
        self.points.iter().copied()
    }
}

impl Region2 {
    pub fn segments<'a>(&'a self) -> impl 'a + Iterator<Item = Segment2> {
        self.loops.iter().flat_map(|l| l.edges()).map(|e| e.map(|i| self.points[i]))
    }

    pub fn contains(&self, point: na::Point2<f64>) -> bool {
        let mut inside = false;
        let ray = Ray2 {
            s: point,
            d: na::Vector2::x(),
        };
        for segment in self.segments() {
            inside ^= segment.ray_intersection(&ray).is_some();
        }

        inside
    }
}

pub enum SegmentRef {
    Orig {
        loop: usize,
        segment: usize,
    },
    Cut {
        i: usize,
        side: bool,
    }
}

pub struct SegmentCut {
    loop: usize,
    segment: usize,
    cut_point: usize,
}

pub struct RegionIntersection2 {
    intersection_points: Vec<na::Point2<f64>>,
    segment_cuts_r1: Vec<SegmentCut>,
    segment_cuts_r2: Vec<SegmentCut>,

    segments_inside_r1: Vec<SegmentRef>,
    segments_inside_r2: Vec<SegmentRef>,
}

impl RegionIntersection2 {
    pub fn compute(r1: &Region1, r2: &Region2) -> Self {
    }
}

pub struct Split<R, V> {
    orig: R,
    new_outside: Vec<V>,
    new_inside: Vec<V>,
} 

struct Intersection<R, V> {
    splits_c1: Vec<CellSplit<R, V>>,
    splits_c2: Vec<CellSplit<R, V>>,
    inside_c1: Vec<R>,
    inside_c2: Vec<R>,
}

pub type CellSplit<R: Region> = Split<R::CellRef, R::Cell>;
pub type RegionIntersection<R: Region> = Intersection<R::CellRef, R::Cell>;

pub type SubRegionSplit<C: Cell> = Split<C::SubRegionRef, C::SubRegion>;
pub type CellIntersection<C: Cell> = Intersection<C::SubRegionRef, C::SubRegion>;

trait SubRegion {
    type Surface;
    type SurfaceIntersection;

    pub fn 
}

trait Cell {
    type SubRegion: SubRegion;
    type SubRegionRef;

    fn subregion(&self, rref: SubRegionRef) -> SubRegion;

    fn contains(&self, other: &Cell) -> bool;

    fn intersects(&self, other: &Cell) -> bool {
        self.intersection(other).is_some()
    }

    fn intersection(&self, other: &Cell) -> Option<CellIntersection<Self>>;
}

pub struct SplitResult<R> {
    new_outside: Vec<R>,
    new_inside: Vec<R>,
} 

trait CellEdit: Cell {
    fn apply_split(&mut self, split: &SubRegionSplit<Self>) -> SplitResult<Self::SubRegionRef>;

    /*
    fn apply_intersection(&mut self, intersection: &CellIntersection<Self>) -> SplitResult<Self::SubRegionRef> {
        let mut res = SplitResult::new();
    }
    */
}

trait Region {
    type Cell: Cell;

    type CellIter: Iterator<Item = CellRef>;

    fn cells(&self) -> CellIter;

    fn cell(&self, cref: CellRef) -> &Cell;

    fn cell_hole(&self, cref: CellRef) -> bool;

    fn intersects_cell(&self, c1: &Self::Cell) -> bool {
        for cref in self.cells() {
            let c2 = self.cell(cref);
            if !self.cell_hole(cref) && c2.contains(c1) {
                return true
            }

            if c2.intersects(c1) {
                return true
            }
        }
    }

    pub fn intersection(&self, other: &Self) -> Option<RegionIntersection<Self>> {
        let mut intersection = Intersection::new();

        for cref in other.cells() 

        if intersection.is_empty() {
            None
        } else {
            Some(intersection)
        }
    }
}
