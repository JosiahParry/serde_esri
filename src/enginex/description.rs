//! Which attributes each vertex carries.

/// A vertex attribute. Discriminants match `VertexDescription.Semantics`.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Semantics {
    Position = 0,
    Z = 1,
    M = 2,
    Id = 3,
}

impl Semantics {
    /// The bit this attribute occupies in a [`VertexDescription`].
    pub const fn bit(self) -> u16 {
        1 << self as u16
    }

    /// Value of this attribute on vertices that do not set it (`VertexDescription._defaultValues`).
    pub const fn default_value(self) -> f64 {
        match self {
            Semantics::M => f64::NAN,
            _ => 0.0,
        }
    }
}

/// Bitmask of the [`Semantics`] each vertex carries (`VertexDescription.m_semanticsBitArray`).
/// [`Semantics::Position`] is always present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexDescription(u16);

impl VertexDescription {
    pub const XY: Self = Self(Semantics::Position.bit());

    pub const fn has(self, semantics: Semantics) -> bool {
        self.0 & semantics.bit() != 0
    }

    pub fn insert(&mut self, semantics: Semantics) {
        self.0 |= semantics.bit();
    }
}

impl Default for VertexDescription {
    fn default() -> Self {
        Self::XY
    }
}

impl FromIterator<Semantics> for VertexDescription {
    fn from_iter<I: IntoIterator<Item = Semantics>>(iter: I) -> Self {
        let mut description = Self::XY;
        for semantics in iter {
            description.insert(semantics);
        }
        description
    }
}

/// Rejects bitmasks with unsupported attributes or without [`Semantics::Position`].
impl TryFrom<u16> for VertexDescription {
    type Error = u16;

    fn try_from(bits: u16) -> Result<Self, Self::Error> {
        let supported = Semantics::Position.bit()
            | Semantics::Z.bit()
            | Semantics::M.bit()
            | Semantics::Id.bit();
        if bits & !supported != 0 || bits & Semantics::Position.bit() == 0 {
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
