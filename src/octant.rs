use std::mem::size_of;

use typenum::{Unsigned, U0, U1, U2, U3, U4, U5, U6, U7};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Octant: u8 {
        const LDF = 0b000;
        const RDF = 0b001;
        const LUF = 0b010;
        const RUF = 0b011;
        const LDB = 0b100;
        const RDB = 0b101;
        const LUB = 0b110;
        const RUB = 0b111;
    }
}

impl Octant {
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

    pub const fn child_offset_df<T>(&self, depth: usize) -> usize {
        if depth == 0 {
            return size_of::<T>();
        }
        let child_size = crate::octree_size::<T>(depth - 1);
        let end_of_current = size_of::<T>();
        let start_of_next = child_size * (self.bits() as usize);
        end_of_current + start_of_next
    }

    pub const fn child_offset_bf<T>(&self, index: usize, depth: usize) -> usize {
        if depth == 0 {
            return size_of::<T>();
        }
        let end_of_current = (8 - index) * 8usize.pow(depth as u32 - 1) * size_of::<T>();
        let start_of_next = index * 8usize.pow(depth as u32) * size_of::<T>();
        end_of_current + start_of_next
    }
}

pub trait OctantT {
    const VALUE: Octant;
    const USIZE: usize;
    type IndexT: Unsigned;
}

pub struct OctantLDF;
impl OctantT for OctantLDF {
    const VALUE: Octant = Octant::LDF;
    const USIZE: usize = 0;
    type IndexT = U0;
}

pub struct OctantRDF;
impl OctantT for OctantRDF {
    const VALUE: Octant = Octant::RDF;
    const USIZE: usize = 1;
    type IndexT = U1;
}

pub struct OctantLUF;
impl OctantT for OctantLUF {
    const VALUE: Octant = Octant::LUF;
    const USIZE: usize = 2;
    type IndexT = U2;
}

pub struct OctantRUF;
impl OctantT for OctantRUF {
    const VALUE: Octant = Octant::RUF;
    const USIZE: usize = 3;
    type IndexT = U3;
}

pub struct OctantLDB;
impl OctantT for OctantLDB {
    const VALUE: Octant = Octant::LDB;
    const USIZE: usize = 4;
    type IndexT = U4;
}

pub struct OctantRDB;
impl OctantT for OctantRDB {
    const VALUE: Octant = Octant::RDB;
    const USIZE: usize = 5;
    type IndexT = U5;
}

pub struct OctantLUB;
impl OctantT for OctantLUB {
    const VALUE: Octant = Octant::LUB;
    const USIZE: usize = 6;
    type IndexT = U6;
}

pub struct OctantRUB;
impl OctantT for OctantRUB {
    const VALUE: Octant = Octant::RUB;
    const USIZE: usize = 7;
    type IndexT = U7;
}
