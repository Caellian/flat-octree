use std::{
    alloc::Layout,
    mem::{align_of, size_of},
};

pub mod octant;
pub mod octree;
pub mod util;

pub use typenum;

pub mod layout {
    use std::mem::size_of;

    use crate::octant::Octant;

    /// A trait for managing different octree memory layouts.
    pub trait MemoryLayout {
        /// Fills the subtree at the given `base` pointer with the given `value`.
        ///
        /// # Safety
        ///
        /// For this function to be safe, the `base` pointer must be valid and
        /// aligned for the given `T` type, the `size` must be the size of
        /// the whole octree, the `depth` must be the (remaining) depth of the
        /// subtree, and the `index` must be the index of `base` node at the
        /// current layer (`size - depth`).
        ///
        /// Additionally, the surrounding layout of `base` must follow the
        /// layout described by the [`MemoryLayout`] implementation.
        unsafe fn fill<T: Clone>(base: *mut T, value: T, size: usize, depth: usize, index: usize);
        /// Returns the offset of the `octant` child from node location described
        /// by:
        /// - `size` - the size of the whole octree,
        /// - `depth` - the (remaining) depth of the subtree,
        /// - `index` - the index of the node at the current (`size - depth`) layer.
        fn child_offset<T>(octant: Octant, size: usize, depth: usize, index: usize) -> usize;
    }

    /// A depth-first memory layout.
    ///
    /// In this layout, octree values are stored such that the first value is
    /// the root octant, which is followed by all first octant children until
    /// the last layer which is tightly packed. After the last layer, the
    /// second-to-last layer second octant value is stored, followed by all of
    /// its children, and so on...
    ///
    /// This representation is better for CPU processing and collision
    /// detection.
    pub struct DepthFirst;
    impl MemoryLayout for DepthFirst {
        unsafe fn fill<T: Clone>(
            base: *mut T,
            value: T,
            _size: usize,
            depth: usize,
            _index: usize,
        ) {
            let tailing = crate::subtree_size::<T>(depth) / size_of::<T>();
            for i in 0..tailing {
                base.add(i).write(value.clone())
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

    /// A breath-first memory layout.
    ///
    /// In this layout, octree values are stored such that every layer values
    /// are stored together, starting from the root layer (1 value), followed by
    /// the first layer (8 values), then third (64 values), and so on...
    ///
    /// This representation is ideal for parallel processing and LOD streaming.
    ///
    /// Additionally, it allows accessing each layer directly as a slice of
    /// memory.
    pub struct BreathFirst;
    impl MemoryLayout for BreathFirst {
        unsafe fn fill<T: Clone>(base: *mut T, value: T, size: usize, depth: usize, index: usize) {
            let height = size - depth;
            let mut start = base;

            for i in 0..=depth {
                let fill_size = crate::layer_length(i);
                for j in 0..fill_size {
                    start.add(j).write(value.clone());
                }

                let layer_i = height + i;
                let layer_size = crate::layer_length(layer_i);
                let end_of_current = layer_size - (index + 1) * fill_size;

                let skip_leading = index * fill_size * 8;
                start = start.add(fill_size + end_of_current + skip_leading);
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

/// Returns a length of an octree layer at the given `depth`.
#[inline(always)]
pub const fn layer_length(depth: usize) -> usize {
    8usize.pow(depth as u32)
}

/// Returns a length of an octree subtree for the given `depth`.
pub const fn subtree_length(depth: usize) -> usize {
    let mut accum = 1;

    let mut i = depth;
    while i > 0 {
        accum *= 8;
        accum += 1;
        i -= 1;
    }

    accum
}

/// Returns a size of an octree subtree for the given `depth`.
pub const fn subtree_size<T>(depth: usize) -> usize {
    subtree_length(depth) * size_of::<T>()
}

/// Returns a [`Layout`] of an octree subtree for the given `depth`.
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
