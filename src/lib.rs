use std::{
    alloc::Layout,
    mem::{align_of, size_of},
};

//pub mod octree;
pub mod octant;
pub mod octree;

pub use typenum;

pub mod layout {
    use std::{mem::size_of, ptr::addr_of_mut};

    use crate::octant::Octant;

    pub trait MemoryLayout {
        fn fill<T: Clone>(base: *mut T, value: T, size: usize, depth: usize, index: usize);
        fn child_offset<T>(octant: Octant, index: usize, depth: usize) -> usize;
    }

    pub struct DepthFirst;
    impl MemoryLayout for DepthFirst {
        fn fill<T: Clone>(base: *mut T, value: T, _size: usize, depth: usize, _index: usize) {
            let tailing = crate::octree_size::<T>(depth) / size_of::<T>();
            for i in 0..tailing {
                unsafe { base.add(i).write(value.clone()) }
            }
        }

        fn child_offset<T>(octant: Octant, _index: usize, depth: usize) -> usize {
            if depth == 0 {
                return size_of::<T>();
            }
            let child_size = crate::octree_size::<T>(depth - 1);
            let end_of_current = size_of::<T>();
            let start_of_next = child_size * (octant.bits() as usize);
            end_of_current + start_of_next
        }
    }

    pub struct BreathFirst;
    impl MemoryLayout for BreathFirst {
        fn fill<T: Clone>(base: *mut T, value: T, size: usize, depth: usize, index: usize) {
            unsafe {
                let start_layer = size - depth;
                let mut start = base;

                for i in 0..=depth {
                    let fill_size = 8usize.pow(i as u32);
                    for j in 0..fill_size {
                        start.add(j).write(value.clone());
                    }

                    let layer_i = start_layer + i;
                    let layer_size = 8usize.pow(layer_i as u32);
                    let end_of_current = (layer_size - (index + 1) * fill_size) * size_of::<T>();

                    let skip_leading = index * fill_size * 8 * size_of::<T>();
                    start = start.add(fill_size + end_of_current + skip_leading);
                }
            }
        }

        fn child_offset<T>(_octant: Octant, index: usize, depth: usize) -> usize {
            if depth == 0 {
                return size_of::<T>();
            }
            let end_of_current = (8 - index) * 8usize.pow(depth as u32 - 1) * size_of::<T>();
            // FIXME: Not using octant?
            let start_of_next = index * 8usize.pow(depth as u32) * size_of::<T>();
            end_of_current + start_of_next
        }
    }

    pub type DF = DepthFirst;
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
