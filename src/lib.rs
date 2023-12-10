use std::{
    alloc::Layout,
    mem::{align_of, size_of},
};

//pub mod octree;
pub mod octant;
pub mod octree_shader;

pub use typenum;

pub mod layout {
    pub trait MemoryLayout {}

    pub struct DepthFirst;
    pub type DF = DepthFirst;

    pub struct BreathFirst;
    pub type BF = BreathFirst;
}

pub const fn octree_size<T>(depth: usize) -> usize {
    let mut accum = size_of::<T>();

    let mut i = depth;
    while i > 0 {
        accum *= 8;
        accum += size_of::<T>();
        i -= 1;
    }

    return accum;
}

pub fn octree_layout<T>(depth: usize) -> Layout {
    Layout::from_size_align(octree_size::<T>(depth), align_of::<T>()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octree_size_test() {
        assert_eq!(octree_size::<u8>(0), 1);
        assert_eq!(octree_size::<u8>(1), 1 + 8 * 1);
        assert_eq!(octree_size::<u8>(2), 1 + 8 * (1 + 8 * 1));
        assert_eq!(octree_size::<u8>(3), 1 + 8 * (1 + 8 * (1 + 8 * 1)));
    }
}
