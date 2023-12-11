use std::{
    alloc::Layout,
    mem::{align_of, size_of},
};

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

/// Provides a way to iterate over children tuple by unrolling the provided body
/// 8 times for each.
#[macro_export]
macro_rules! for_each_child {
    ($name: ident: $children: expr => $body: block) => {{
        let children__ = $children;
        {
            let $name = children__.0;
            $body
        }
        {
            let $name = children__.1;
            $body
        }
        {
            let $name = children__.2;
            $body
        }
        {
            let $name = children__.3;
            $body
        }
        {
            let $name = children__.4;
            $body
        }
        {
            let $name = children__.5;
            $body
        }
        {
            let $name = children__.6;
            $body
        }
        {
            let $name = children__.7;
            $body
        }
    }};
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
