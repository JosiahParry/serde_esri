//! Single vertices and column-wise vertex storage.

use crate::enginex::{Semantics, VertexDescription};

/// A single vertex with all of its attributes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub m: Option<f64>,
    pub id: Option<i32>,
}

impl Vertex {
    pub fn description(&self) -> VertexDescription {
        [
            self.z.map(|_| Semantics::Z),
            self.m.map(|_| Semantics::M),
            self.id.map(|_| Semantics::Id),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// Column-wise vertex storage. Every present attribute column has the same length as `xy`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VertexAttributes {
    pub xy: Vec<[f64; 2]>,
    pub z: Option<Vec<f64>>,
    pub m: Option<Vec<f64>>,
    pub id: Option<Vec<i32>>,
}

impl VertexAttributes {
    pub fn len(&self) -> usize {
        self.xy.len()
    }

    pub fn is_empty(&self) -> bool {
        self.xy.is_empty()
    }

    pub fn description(&self) -> VertexDescription {
        [
            self.z.as_ref().map(|_| Semantics::Z),
            self.m.as_ref().map(|_| Semantics::M),
            self.id.as_ref().map(|_| Semantics::Id),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn get(&self, index: usize) -> Option<Vertex> {
        let [x, y] = *self.xy.get(index)?;
        Some(Vertex {
            x,
            y,
            z: self.z.as_ref().and_then(|z| z.get(index).copied()),
            m: self.m.as_ref().and_then(|m| m.get(index).copied()),
            id: self.id.as_ref().and_then(|id| id.get(index).copied()),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = Vertex> + '_ {
        (0..self.len()).filter_map(|i| self.get(i))
    }
}

/// Columns follow the first vertex: an attribute it carries is kept for all, with the engine default where missing.
impl FromIterator<Vertex> for VertexAttributes {
    fn from_iter<I: IntoIterator<Item = Vertex>>(vertices: I) -> Self {
        let mut vertices = vertices.into_iter().peekable();
        let first = vertices.peek().copied().unwrap_or_default();
        let mut columns = VertexAttributes {
            xy: Vec::new(),
            z: first.z.map(|_| Vec::new()),
            m: first.m.map(|_| Vec::new()),
            id: first.id.map(|_| Vec::new()),
        };
        for v in vertices {
            columns.xy.push([v.x, v.y]);
            if let Some(z) = &mut columns.z {
                z.push(v.z.unwrap_or(Semantics::Z.default_value()));
            }
            if let Some(m) = &mut columns.m {
                m.push(v.m.unwrap_or(Semantics::M.default_value()));
            }
            if let Some(id) = &mut columns.id {
                id.push(v.id.unwrap_or(0));
            }
        }
        columns
    }
}
