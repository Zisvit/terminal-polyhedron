//! Polyhedron geometry: vertices and edges for supported shapes (wireframe).

use crate::math::Vec3;

/// A polyhedron: its vertices and edges (wireframe model).
pub struct Polyhedron {
    pub vertices: Vec<Vec3>,
    pub edges: Vec<(usize, usize)>,
}

impl Polyhedron {
    /// Unit cube centered at origin. 8 vertices, 12 edges.
    pub fn cube() -> Self {
        let v = [
            Vec3::new(-1.0, -1.0, -1.0), // 0
            Vec3::new(1.0, -1.0, -1.0),  // 1
            Vec3::new(1.0, 1.0, -1.0),   // 2
            Vec3::new(-1.0, 1.0, -1.0),  // 3
            Vec3::new(-1.0, -1.0, 1.0),  // 4
            Vec3::new(1.0, -1.0, 1.0),   // 5
            Vec3::new(1.0, 1.0, 1.0),    // 6
            Vec3::new(-1.0, 1.0, 1.0),   // 7
        ];

        let edges = vec![
            // back face
            (0, 1), (1, 2), (2, 3), (3, 0),
            // front face
            (4, 5), (5, 6), (6, 7), (7, 4),
            // connecting edges
            (0, 4), (1, 5), (2, 6), (3, 7),
        ];

        Self {
            vertices: v.to_vec(),
            edges,
        }
    }

    /// Regular icosahedron: 12 vertices, 30 edges.
    pub fn icosahedron() -> Self {
        let t = (1.0 + 5.0_f64.sqrt()) / 2.0; // golden ratio
        let v = [
            Vec3::new(-1.0, t, 0.0),
            Vec3::new(1.0, t, 0.0),
            Vec3::new(-1.0, -t, 0.0),
            Vec3::new(1.0, -t, 0.0),
            Vec3::new(0.0, -1.0, t),
            Vec3::new(0.0, 1.0, t),
            Vec3::new(0.0, -1.0, -t),
            Vec3::new(0.0, 1.0, -t),
            Vec3::new(t, 0.0, -1.0),
            Vec3::new(t, 0.0, 1.0),
            Vec3::new(-t, 0.0, -1.0),
            Vec3::new(-t, 0.0, 1.0),
        ];

        // Edge length of this icosahedron is 2.
        let edges = compute_edges(&v, 2.0);

        Self {
            vertices: v.to_vec(),
            edges,
        }
    }

    /// Regular dodecahedron: 20 vertices, 30 edges, 12 pentagonal faces.
    ///
    /// Vertices: (±1,±1,±1), (0,±1/φ,±φ), (±1/φ,±φ,0), (±φ,0,±1/φ).
    /// Edge length: 2/φ.
    pub fn dodecahedron() -> Self {
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let inv_phi = 1.0 / phi;

        let mut v = Vec::with_capacity(20);
        // (±1, ±1, ±1)
        for sx in [-1.0, 1.0] {
            for sy in [-1.0, 1.0] {
                for sz in [-1.0, 1.0] {
                    v.push(Vec3::new(sx, sy, sz));
                }
            }
        }
        // (0, ±1/φ, ±φ)
        for sy in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                v.push(Vec3::new(0.0, sy * inv_phi, sz * phi));
            }
        }
        // (±1/φ, ±φ, 0)
        for sx in [-1.0, 1.0] {
            for sy in [-1.0, 1.0] {
                v.push(Vec3::new(sx * inv_phi, sy * phi, 0.0));
            }
        }
        // (±φ, 0, ±1/φ)
        for sx in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                v.push(Vec3::new(sx * phi, 0.0, sz * inv_phi));
            }
        }

        assert_eq!(v.len(), 20);

        // Edge length of dodecahedron with these coordinates is 2/φ.
        let edge_len = 2.0 / phi;
        let edges = compute_edges(&v, edge_len);

        assert_eq!(edges.len(), 30);

        Self { vertices: v, edges }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "cube" => Some(Self::cube()),
            "icosahedron" => Some(Self::icosahedron()),
            "dodecahedron" => Some(Self::dodecahedron()),
            _ => None,
        }
    }
}

/// Compute edges by connecting all vertex pairs at approximately `target_len` distance.
fn compute_edges(vertices: &[Vec3], target_len: f64) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    let tol = 1e-6;
    for i in 0..vertices.len() {
        for j in (i + 1)..vertices.len() {
            let d = (vertices[i] - vertices[j]).length();
            if (d - target_len).abs() < tol {
                edges.push((i, j));
            }
        }
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_has_8_vertices_and_12_edges() {
        let c = Polyhedron::cube();
        assert_eq!(c.vertices.len(), 8);
        assert_eq!(c.edges.len(), 12);
    }

    #[test]
    fn icosahedron_has_12_vertices_and_30_edges() {
        let i = Polyhedron::icosahedron();
        assert_eq!(i.vertices.len(), 12);
        assert_eq!(i.edges.len(), 30);
    }

    #[test]
    fn dodecahedron_has_20_vertices_and_30_edges() {
        let d = Polyhedron::dodecahedron();
        assert_eq!(d.vertices.len(), 20);
        assert_eq!(d.edges.len(), 30);
    }

    #[test]
    fn dodecahedron_edges_connect_correct_distance() {
        let d = Polyhedron::dodecahedron();
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let edge_len = 2.0 / phi;
        for &(a, b) in &d.edges {
            let dist = (d.vertices[a] - d.vertices[b]).length();
            assert!((dist - edge_len).abs() < 1e-6);
        }
    }

    #[test]
    fn dodecahedron_is_3_regular() {
        let d = Polyhedron::dodecahedron();
        let mut degree = [0usize; 20];
        for &(a, b) in &d.edges {
            degree[a] += 1;
            degree[b] += 1;
        }
        for &deg in &degree {
            assert_eq!(deg, 3);
        }
    }
}
