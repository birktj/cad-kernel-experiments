use crate::geometry::{LinePoint, EPSILON};

pub type Point1D = LinePoint;

/// A 1 dimensional boundary region.
///
/// A 1 dimensional region is just a collection of line points without any additional 
/// topological structure.
#[derive(Clone, Debug)]
pub struct Region1D {
    points: Vec<Point1D>,
}

impl Region1D {
    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
        }
    }

    pub fn new(mut points: Vec<Point1D>) -> Option<Self> {
        points.sort_by(|a, b| a.pos.total_cmp(&b.pos).then(a.dir.cmp(&b.dir)));

        // First direction must be positive
        let mut dir = false;
        let mut pos = None;

        for p in &points {
            // Dirs must be different
            if p.dir == dir {
                return None
            }

            dir = p.dir;
            pos = Some(p.pos);
        }

        // Last direction must be negative
        if dir {
            return None
        }

        Some(Self {
            points
        })
    }

    pub fn cells(&self) -> impl Iterator<Item = [usize; 2]> {
        (0..self.points.len()).step_by(2).map(|i| [i, i+1])
    }

    pub fn points(&self) -> &[Point1D] {
        &self.points
    }

    pub fn inside(&self, point: f64) -> bool {
        match self.points.binary_search_by(|x| x.pos.total_cmp(&point).then(x.dir.cmp(&true))) {
            Ok(_) => true,
            Err(i) if i > 0 && i < self.points.len() => self.points[i-1].dir,
            Err(_) => false,
        }
    }

    pub fn contains(&self, other: &Region1D) -> bool {
        self.intersection(other).r1_inside.is_empty()
    }

    pub fn intersection(&self, other: &Region1D) -> RegionIntersection1D {
        let r1_inside = self.points().iter().enumerate()
            .filter(|(i, p)| other.inside(p.pos))
            .map(|p| p.0).collect();

        let r2_inside = other.points().iter().enumerate()
            .filter(|(i, p)| self.inside(p.pos))
            .map(|p| p.0).collect();

        RegionIntersection1D {
            r1_inside,
            r2_inside,
        }
    }
}

pub struct RegionIntersection1D {
    pub r1_inside: Vec<usize>,
    pub r2_inside: Vec<usize>,
}
