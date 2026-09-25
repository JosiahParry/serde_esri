//! Which attributes each vertex carries.

/// A vertex attribute, mirroring the engine's `VertexDescription.Semantics` with matching discriminants.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Attribute {
    Position = 0,
    Z = 1,
    M = 2,
    Id = 3,
}

impl Attribute {
    /// The bit this attribute occupies in a [`VertexDescription`].
    pub const fn bit(self) -> u16 {
        1 << self as u16
    }

    /// Value of this attribute on vertices that do not set it (`VertexDescription._defaultValues`).
    pub const fn default_value(self) -> f64 {
        match self {
            Attribute::M => f64::NAN,
            _ => 0.0,
        }
    }
}

/// Bitmask of the [`Attribute`]s each vertex carries (`VertexDescription.m_semanticsBitArray`).
/// [`Attribute::Position`] is always present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexDescription(u16);

impl VertexDescription {
    pub const XY: Self = Self(Attribute::Position.bit());

    pub const fn has(self, attribute: Attribute) -> bool {
        self.0 & attribute.bit() != 0
    }

    pub fn insert(&mut self, attribute: Attribute) {
        self.0 |= attribute.bit();
    }
}

impl Default for VertexDescription {
    fn default() -> Self {
        Self::XY
    }
}

impl FromIterator<Attribute> for VertexDescription {
    fn from_iter<I: IntoIterator<Item = Attribute>>(iter: I) -> Self {
        let mut description = Self::XY;
        for attribute in iter {
            description.insert(attribute);
        }
        description
    }
}

/// Rejects bitmasks with unsupported attributes or without [`Attribute::Position`].
impl TryFrom<u16> for VertexDescription {
    type Error = u16;

    fn try_from(bits: u16) -> Result<Self, Self::Error> {
        let supported = Attribute::Position.bit()
            | Attribute::Z.bit()
            | Attribute::M.bit()
            | Attribute::Id.bit();
        if bits & !supported != 0 || bits & Attribute::Position.bit() == 0 {
            Err(bits)
        } else {
            Ok(Self(bits))
        }
    }
}

impl From<VertexDescription> for u16 {
    fn from(description: VertexDescription) -> Self {
        description.0
    }
}
