#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<na::Point3<f32>>,
    pub faces: Vec<[usize; 3]>,
}

impl Mesh {
    pub fn empty() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn from_raw_tri(tri: [na::Point3<f32>; 3]) -> Mesh {
        Mesh {
            vertices: tri.into(),
            faces: vec![[0,1,2]],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.faces.is_empty()
    }

    pub fn area(&self) -> f32 {
        self.faces.iter()
            .map(|[a, b, c]| {
                let a = self.vertices[*a];
                let b = self.vertices[*b];
                let c = self.vertices[*c];
                (b - a).cross(&(c - a)).norm() / 2.0
            })
            .sum()
    }

    pub fn center(&self) -> na::Point3<f32> {
        let (a, b) = self.bounding_box();
        na::Point3::from((a.coords + b.coords) / 2.0)
    }

    pub fn bounding_box(&self) -> (na::Point3<f32>, na::Point3<f32>) {
        if self.vertices.is_empty() {
            return (na::Point3::origin(), na::Point3::origin())
        }

        let min_vec = self.vertices.iter()
            .map(|p| p.coords)
            .fold(self.vertices[0].coords, |acc, x| acc.inf(&x));
        let max_vec = self.vertices.iter()
            .map(|p| p.coords)
            .fold(self.vertices[0].coords, |acc, x| acc.sup(&x));

        (na::Point3::from(min_vec), na::Point3::from(max_vec))
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn transform(&self, transform: na::Transform3<f32>) -> Mesh {
        let vertices = self.vertices.iter()
            .map(|v| transform * v).collect();

        Mesh {
            vertices,
            faces: self.faces.clone(),
        }
    }

    pub fn faces<'a>(&'a self) -> impl 'a + Iterator<Item = [na::Point3<f32>; 3]> {
        self.faces.iter().map(move |f| f.map(|i| self.vertices[i]))
    }

    pub fn extend(&mut self, other: &Mesh) {
        let vc = self.vertices.len();
        self.vertices.extend_from_slice(&other.vertices);
        self.faces.extend(other.faces.iter()
            .map(|[a, b, c]| [a+vc, b+vc, c+vc]));
    }

    pub fn cube(size: f32) -> Self {
        let mut vertices = Vec::new();
        for x in &[-1.0, 1.0] {
            for y in &[-1.0, 1.0] {
                for z in &[-1.0, 1.0] {
                    vertices.push(na::Point3::new(x*size/2.0, y*size/2.0, z*size/2.0));
                }
            }
        }
        
        let faces = vec![
            [0, 2, 1],
            [3, 1, 2],
            [0, 1, 4],
            [1, 5, 4],
            [4, 5, 6],
            [5, 7, 6],
            [7, 3, 6],
            [2, 6, 3],
            [1, 3, 5],
            [7, 5, 3],
            [0, 4, 2],
            [6, 2, 4],
        ];

        Self {
            vertices,
            faces,
        }
    }

    /*
    pub fn to_ncollide(self) -> ncollide3d::shape::TriMesh<f32> {
        let indices = self.faces.into_iter()
            .map(|[a, b, c]| na::Point3::new(a, b, c))
            .collect();

        ncollide3d::shape::TriMesh::new(
            self.vertices,
            indices, None)
    }
    */
}

