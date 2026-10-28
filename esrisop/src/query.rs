use serde_esri::enginex::geometry::Envelope2D;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    pub extents: Vec<Envelope2D>,
    pub resolution: Option<f64>,
    pub columns: Option<Vec<String>>,
}
