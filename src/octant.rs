use typenum::{Unsigned, U0, U1, U2, U3, U4, U5, U6, U7};

/// Octree octant values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Octant {
    /// Octree left-down-front octant
    LDF = 0b000,
    /// Octree right-down-front octant
    RDF = 0b001,
    /// Octree left-up-front octant
    LUF = 0b010,
    /// Octree right-up-front octant
    RUF = 0b011,
    /// Octree left-down-back octant
    LDB = 0b100,
    /// Octree right-down-back octant
    RDB = 0b101,
    /// Octree left-up-back octant
    LUB = 0b110,
    /// Octree right-up-back octant
    RUB = 0b111,
}

impl Octant {
    /// Collection of all valid octant values.
    pub const ALL: [Octant; 8] = [
        Octant::LDF,
        Octant::RDF,
        Octant::LUF,
        Octant::RUF,
        Octant::LDB,
        Octant::RDB,
        Octant::LUB,
        Octant::RUB,
    ];

    /// Returns the octant value as a `usize`.
    pub const fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl TryFrom<u8> for Octant {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b000..=0b111 => Ok(unsafe {
                // SAFETY: `value` is in the range `0b000..=0b111`.
                std::mem::transmute(value)
            }),
            _ => Err(()),
        }
    }
}
impl TryFrom<usize> for Octant {
    type Error = ();

    #[inline(always)]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u8)
    }
}

/// A trait for type representations of an octant.
pub trait OctantT {
    /// The octant value.
    const VALUE: Octant;
    /// The octant `usize` value.
    const USIZE: usize;
    /// The octant compile-time integer value.
    type IndexT: Unsigned;
}

/// Type representation of [`Octant::LDF`] (left-down-front octant).
pub struct OctantLDF;
impl OctantT for OctantLDF {
    const VALUE: Octant = Octant::LDF;
    const USIZE: usize = 0;
    type IndexT = U0;
}

/// Type representation of [`Octant::RDF`] (right-down-front octant).
pub struct OctantRDF;
impl OctantT for OctantRDF {
    const VALUE: Octant = Octant::RDF;
    const USIZE: usize = 1;
    type IndexT = U1;
}

/// Type representation of [`Octant::LUF`] (left-up-front octant).
pub struct OctantLUF;
impl OctantT for OctantLUF {
    const VALUE: Octant = Octant::LUF;
    const USIZE: usize = 2;
    type IndexT = U2;
}

/// Type representation of [`Octant::RUF`] (right-up-front octant).
pub struct OctantRUF;
impl OctantT for OctantRUF {
    const VALUE: Octant = Octant::RUF;
    const USIZE: usize = 3;
    type IndexT = U3;
}

/// Type representation of [`Octant::LDB`] (left-down-back octant).
pub struct OctantLDB;
impl OctantT for OctantLDB {
    const VALUE: Octant = Octant::LDB;
    const USIZE: usize = 4;
    type IndexT = U4;
}

/// Type representation of [`Octant::RDB`] (right-down-back octant).
pub struct OctantRDB;
impl OctantT for OctantRDB {
    const VALUE: Octant = Octant::RDB;
    const USIZE: usize = 5;
    type IndexT = U5;
}

/// Type representation of [`Octant::LUB`] (left-up-back octant).
pub struct OctantLUB;
impl OctantT for OctantLUB {
    const VALUE: Octant = Octant::LUB;
    const USIZE: usize = 6;
    type IndexT = U6;
}

/// Type representation of [`Octant::RUB`] (right-up-back octant).
pub struct OctantRUB;
impl OctantT for OctantRUB {
    const VALUE: Octant = Octant::RUB;
    const USIZE: usize = 7;
    type IndexT = U7;
}
