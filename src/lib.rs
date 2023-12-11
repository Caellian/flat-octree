#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// Octree octant values and types.
pub mod octant;

mod octree;
pub use octree::*;

/// Octree utility functions.
pub mod util;

pub use typenum;

/// Octree memory layouts.
pub mod layout {
    use std::mem::size_of;

    use crate::octant::Octant;

    /// A trait for managing different octree memory layouts.
    pub trait MemoryLayout {
        /// Fills the subtree at the given `base` pointer with the given
        /// `value`.
        ///
        /// # Safety
        ///
        /// For this function to be safe, the `base` pointer must be valid and
        /// aligned for the given `T` type, the `size` must be the size of the
        /// whole octree, the `depth` must be the (remaining) depth of the
        /// subtree, and the `index` must be the index of `base` node at the
        /// current layer (`size - depth`).
        ///
        /// Additionally, the surrounding layout of `base` must follow the
        /// layout described by the [`MemoryLayout`] implementation.
        unsafe fn fill<T: Clone>(base: *mut T, value: T, size: usize, depth: usize, index: usize);
        /// Returns the offset of the `octant` child from node location
        /// described by:
        /// - `size` - the size of the whole octree,
        /// - `depth` - the (remaining) depth of the subtree,
        /// - `index` - the index of the node at the current (`size - depth`)
        ///   layer.
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
            let tailing = crate::util::subtree_size::<T>(depth) / size_of::<T>();
            for i in 0..tailing {
                base.add(i).write(value.clone())
            }
        }

        fn child_offset<T>(octant: Octant, _size: usize, depth: usize, _index: usize) -> usize {
            if depth == 0 {
                return 1;
            }
            let end_of_current = 1;
            let start_of_next = crate::util::subtree_length(depth - 1) * octant.as_usize();
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
                let fill_size = crate::util::layer_length(i);
                for j in 0..fill_size {
                    start.add(j).write(value.clone());
                }

                let layer_i = height + i;
                let layer_size = crate::util::layer_length(layer_i);
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
            let layer_size = crate::util::layer_length(height);

            let end_of_current = layer_size - index;
            let start_of_next = index * 8 + octant.as_usize();
            end_of_current + start_of_next
        }
    }

    /// A shorthand type alias for [`DepthFirst`].
    pub type DF = DepthFirst;
    /// A shorthand type alias for [`BreathFirst`].
    pub type BF = BreathFirst;
}
pub use layout::{BreathFirst, DepthFirst, BF, DF};
