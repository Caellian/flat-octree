use std::{
    alloc::Layout,
    mem::{align_of, size_of},
};

//pub mod octree;
pub mod octant;
pub mod octree;

pub use typenum;

pub mod layout {
    use std::mem::size_of;

    use crate::octant::Octant;

    pub trait MemoryLayout {
        fn fill<T: Clone>(base: *mut T, value: T, size: usize, depth: usize, index: usize);
        fn child_offset<T>(octant: Octant, size: usize, depth: usize, index: usize) -> usize;
    }

    pub struct DepthFirst;
    impl MemoryLayout for DepthFirst {
        fn fill<T: Clone>(base: *mut T, value: T, _size: usize, depth: usize, _index: usize) {
            let tailing = crate::subtree_size::<T>(depth) / size_of::<T>();
            for i in 0..tailing {
                unsafe { base.add(i).write(value.clone()) }
            }
        }

        fn child_offset<T>(octant: Octant, _size: usize, depth: usize, _index: usize) -> usize {
            if depth == 0 {
                return 1;
            }
            let end_of_current = 1;
            let start_of_next = crate::subtree_length(depth - 1) * octant.as_usize();
            end_of_current + start_of_next
        }
    }

    pub struct BreathFirst;
    impl MemoryLayout for BreathFirst {
        fn fill<T: Clone>(base: *mut T, value: T, size: usize, depth: usize, index: usize) {
            unsafe {
                let height = size - depth;
                let mut start = base;

                for i in 0..=depth {
                    let fill_size = crate::layer_length(i);
                    for j in 0..fill_size {
                        start.add(j).write(value.clone());
                    }

                    let layer_i = height + i;
                    let layer_size = crate::layer_length(layer_i);
                    let end_of_current = (layer_size - (index + 1) * fill_size) * size_of::<T>();

                    let skip_leading = index * fill_size * 8 * size_of::<T>();
                    start = start.add(fill_size + end_of_current + skip_leading);
                }
            }
        }

        fn child_offset<T>(octant: Octant, size: usize, depth: usize, index: usize) -> usize {
            if depth == 0 {
                return size_of::<T>();
            }
            let height = size - depth;
            let layer_size = crate::layer_length(height);

            let end_of_current = layer_size - index;
            let start_of_next = index * 8 + octant.as_usize();
            end_of_current + start_of_next
        }
    }

    pub type DF = DepthFirst;
    pub type BF = BreathFirst;
}

#[inline(always)]
pub const fn layer_length(depth: usize) -> usize {
    8usize.pow(depth as u32)
}

pub const fn subtree_length(depth: usize) -> usize {
    let mut accum = 1;

    let mut i = depth;
    while i > 0 {
        accum *= 8;
        accum += 1;
        i -= 1;
    }

    return accum;
}

pub const fn subtree_size<T>(depth: usize) -> usize {
    return subtree_length(depth) * size_of::<T>();
}

pub fn subtree_layout<T>(depth: usize) -> Layout {
    Layout::from_size_align(subtree_size::<T>(depth), align_of::<T>()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octree_size_test() {
        assert_eq!(subtree_size::<u8>(0), 1);
        assert_eq!(subtree_size::<u8>(1), 1 + 8 * 1);
        assert_eq!(subtree_size::<u8>(2), 1 + 8 * (1 + 8 * 1));
        assert_eq!(subtree_size::<u8>(3), 1 + 8 * (1 + 8 * (1 + 8 * 1)));
    }
}
