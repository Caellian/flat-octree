use std::{
    alloc::Layout,
    marker::PhantomData,
    mem::{forget, size_of},
    ops::{Add, Deref, DerefMut, Mul, Sub},
    ptr::{addr_of, addr_of_mut, null_mut},
};

use typenum::{
    op, IsLess, IsLessOrEqual, Le, LeEq, Same, Sub1, True, Unsigned, U0, U1, U2, U3, U4, U5, U6,
    U7, U8,
};

use crate::{
    layout::{BreathFirst, OctreeLayout},
    octant::*,
    util::{subtree_length, subtree_size},
};

mod sealed {
    use typenum::{NonZero, Unsigned, B1, U8};

    use super::*;

    pub trait NotLast: Unsigned + NonZero + Sub<B1>
    where
        <Self as Sub<B1>>::Output: Unsigned,
    {
    }
    impl<N: Unsigned + NonZero + Sub<B1>> NotLast for N where <N as Sub<B1>>::Output: Unsigned {}

    pub trait IndexChild<ChildOctant: OctantT>: Unsigned + Mul<U8>
    where
        <Self as Mul<U8>>::Output: Add<ChildOctant::IndexT>,
        <<Self as Mul<U8>>::Output as Add<ChildOctant::IndexT>>::Output: Unsigned,
    {
    }
    impl<N: Unsigned + Mul<U8>, ChildOctant: OctantT> IndexChild<ChildOctant> for N
    where
        <N as Mul<U8>>::Output: Add<ChildOctant::IndexT>,
        <<N as Mul<U8>>::Output as Add<ChildOctant::IndexT>>::Output: Unsigned,
    {
    }
}

type ChildIndex<I, ChildOctant> = <op!(I * U8) as Add<<ChildOctant as OctantT>::IndexT>>::Output;

/// Utility type alias for [`OctreeNode::children`] result.
pub type ChildrenRef<'a, T, Size, L, Depth, Index> = (
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLDF>>,
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRDF>>,
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLUF>>,
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRUF>>,
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLDB>>,
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRDB>>,
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLUB>>,
    &'a OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRUB>>,
);
/// Utility type alias for [`OctreeNode::children_mut`] result.
pub type ChildrenRefMut<'a, T, Size, L, Depth, Index> = (
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLDF>>,
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRDF>>,
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLUF>>,
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRUF>>,
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLDB>>,
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRDB>>,
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantLUB>>,
    &'a mut OctreeNode<T, Size, L, Sub1<Depth>, ChildIndex<Index, OctantRUB>>,
);

/// Octree node structure.
///
/// Shouldn't be constructed/dropped directly, use [`Octree`] instead - calling
/// a drop on this type will result in UB.
#[derive(Debug)]
#[repr(transparent)]
pub struct OctreeNode<
    T: Clone,
    Size: Unsigned,
    L: OctreeLayout,
    Depth: Unsigned = Size,
    LayerIndex: Unsigned = U0,
> {
    value: T,
    _phantom: PhantomData<(L, Size, Depth, LayerIndex)>,
}

impl<T: Clone, S: Unsigned, L: OctreeLayout, D: Unsigned, I: Unsigned> OctreeNode<T, S, L, D, I> {
    /// Returns the current node octant relative to parent.
    pub const fn octant(&self) -> Octant
    where
        D: IsLess<S>,
        Le<D, S>: Same<True>,
    {
        Octant::ALL[I::USIZE % 8]
    }

    /// Returns the node value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Sets the `value` of this node as well as its descendants.
    pub fn set_value(&mut self, value: T) {
        unsafe {
            L::fill(
                addr_of_mut!(self.value),
                value,
                S::USIZE,
                D::USIZE,
                I::USIZE,
            )
        }
    }

    /// Returns the child node at the given `octant`.
    pub fn child<ChildOctant: OctantT>(
        &self,
    ) -> &OctreeNode<T, S, L, Sub1<D>, ChildIndex<I, ChildOctant>>
    where
        D: sealed::NotLast,
        Sub1<D>: Unsigned,
        I: sealed::IndexChild<ChildOctant>,
        <I as Mul<U8>>::Output: Add<ChildOctant::IndexT>,
        <<I as Mul<U8>>::Output as Add<ChildOctant::IndexT>>::Output: Unsigned,
    {
        unsafe {
            let pos = addr_of!(self.value).add(L::child_offset::<T>(
                ChildOctant::VALUE,
                S::USIZE,
                D::USIZE,
                I::USIZE,
            ))
                as *const OctreeNode<T, S, L, Sub1<D>, ChildIndex<I, ChildOctant>>;
            pos.as_ref().unwrap_unchecked()
        }
    }

    /// Returns the mutable child node at the given `octant`.
    pub fn child_mut<ChildOctant: OctantT>(
        &mut self,
    ) -> &mut OctreeNode<T, S, L, Sub1<D>, ChildIndex<I, ChildOctant>>
    where
        D: sealed::NotLast,
        Sub1<D>: Unsigned,
        I: sealed::IndexChild<ChildOctant>,
        <I as Mul<U8>>::Output: Add<ChildOctant::IndexT>,
        <<I as Mul<U8>>::Output as Add<ChildOctant::IndexT>>::Output: Unsigned,
    {
        unsafe {
            let pos = addr_of_mut!(self.value).add(L::child_offset::<T>(
                ChildOctant::VALUE,
                S::USIZE,
                D::USIZE,
                I::USIZE,
            ))
                as *mut OctreeNode<T, S, L, Sub1<D>, ChildIndex<I, ChildOctant>>;
            pos.as_mut().unwrap_unchecked()
        }
    }

    /// Propagates most frequent subtree values from bottom to the top.
    ///
    /// This is a no-op implementation when the subtree depth is 0 (a
    /// single/leaf value).
    pub fn propagate_common(&mut self)
    where
        T: PartialEq,
    {
        #[inline(always)]
        unsafe fn child_ref<'a, T: Clone, S: Unsigned, L: OctreeLayout>(
            base: &T,
            octant: usize,
            depth: usize,
            index: usize,
        ) -> &'a T {
            let pos = addr_of!(*base).add(L::child_offset::<T>(
                Octant::try_from(octant as u8).unwrap_unchecked(),
                S::USIZE,
                depth,
                index,
            )) as *const T;
            pos.as_ref().unwrap_unchecked()
        }

        unsafe fn propagate_layer<T: Clone + PartialEq, S: Unsigned, L: OctreeLayout>(
            base: *mut T,
            layer_depth: usize,
            layer_index: usize,
        ) {
            // No need to balance if the subtree depth is 0
            if layer_depth == 0 {
                return;
            }

            // TODO: recursion required for value to be correct
            let value = base.as_mut().unwrap_unchecked();
            let mut counts = [0u8; 8];
            'outer: for child_i in 0..8 {
                let child = child_ref::<T, S, L>(&value, child_i, layer_depth, layer_index);

                // TODO: If Hash this is implemented inner loop can be a hash lookup
                for compared_i in 0..child_i {
                    let other = child_ref::<T, S, L>(&value, child_i, layer_depth, layer_index);
                    if matches!(child.eq(other), true) {
                        counts[compared_i] += 1;
                        continue 'outer;
                    }
                }
                counts[child_i] += 1;
            }

            let largest_i = counts
                .iter()
                .enumerate()
                .max_by_key(|it| it.1)
                .map(|it| it.0)
                .unwrap();

            let largest = child_ref::<T, S, L>(&value, largest_i, layer_depth, layer_index);
            *value = largest.clone();
        }

        unsafe { propagate_layer::<T, S, L>(&mut self.value, D::USIZE, I::USIZE) }
    }

    /// Returns a tuple of all the children nodes.
    pub fn children<'a>(&'a self) -> ChildrenRef<'a, T, S, L, D, I>
    where
        D: sealed::NotLast,
        Sub1<D>: Unsigned,
        I: sealed::IndexChild<OctantLDF>,
        I: sealed::IndexChild<OctantRDF>,
        I: sealed::IndexChild<OctantLUF>,
        I: sealed::IndexChild<OctantRUF>,
        I: sealed::IndexChild<OctantLDB>,
        I: sealed::IndexChild<OctantRDB>,
        I: sealed::IndexChild<OctantLUB>,
        I: sealed::IndexChild<OctantRUB>,
        <I as Mul<U8>>::Output: Add<U0>,
        <I as Mul<U8>>::Output: Add<U1>,
        <I as Mul<U8>>::Output: Add<U2>,
        <I as Mul<U8>>::Output: Add<U3>,
        <I as Mul<U8>>::Output: Add<U4>,
        <I as Mul<U8>>::Output: Add<U5>,
        <I as Mul<U8>>::Output: Add<U6>,
        <I as Mul<U8>>::Output: Add<U7>,
        op!(I * U8 + U0): Unsigned,
        op!(I * U8 + U1): Unsigned,
        op!(I * U8 + U2): Unsigned,
        op!(I * U8 + U3): Unsigned,
        op!(I * U8 + U4): Unsigned,
        op!(I * U8 + U5): Unsigned,
        op!(I * U8 + U6): Unsigned,
        op!(I * U8 + U7): Unsigned,
    {
        (
            self.child::<OctantLDF>(),
            self.child::<OctantRDF>(),
            self.child::<OctantLUF>(),
            self.child::<OctantRUF>(),
            self.child::<OctantLDB>(),
            self.child::<OctantRDB>(),
            self.child::<OctantLUB>(),
            self.child::<OctantRUB>(),
        )
    }

    /// Returns a mutable tuple of all the children nodes.
    pub fn children_mut<'a>(&'a mut self) -> ChildrenRefMut<'a, T, S, L, D, I>
    where
        D: sealed::NotLast,
        Sub1<D>: Unsigned,
        I: sealed::IndexChild<OctantLDF>,
        I: sealed::IndexChild<OctantRDF>,
        I: sealed::IndexChild<OctantLUF>,
        I: sealed::IndexChild<OctantRUF>,
        I: sealed::IndexChild<OctantLDB>,
        I: sealed::IndexChild<OctantRDB>,
        I: sealed::IndexChild<OctantLUB>,
        I: sealed::IndexChild<OctantRUB>,
        <I as Mul<U8>>::Output: Add<U0>,
        <I as Mul<U8>>::Output: Add<U1>,
        <I as Mul<U8>>::Output: Add<U2>,
        <I as Mul<U8>>::Output: Add<U3>,
        <I as Mul<U8>>::Output: Add<U4>,
        <I as Mul<U8>>::Output: Add<U5>,
        <I as Mul<U8>>::Output: Add<U6>,
        <I as Mul<U8>>::Output: Add<U7>,
        <<I as Mul<U8>>::Output as Add<U0>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U1>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U2>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U3>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U4>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U5>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U6>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U7>>::Output: Unsigned,
    {
        unsafe {
            // SAFETY: as child data isn't overlapping, it's safe to split &mut
            // self into 8 mutable references of all the children
            (
                addr_of_mut!(*self.child_mut::<OctantLDF>())
                    .as_mut()
                    .unwrap_unchecked(),
                addr_of_mut!(*self.child_mut::<OctantRDF>())
                    .as_mut()
                    .unwrap_unchecked(),
                addr_of_mut!(*self.child_mut::<OctantLUF>())
                    .as_mut()
                    .unwrap_unchecked(),
                addr_of_mut!(*self.child_mut::<OctantRUF>())
                    .as_mut()
                    .unwrap_unchecked(),
                addr_of_mut!(*self.child_mut::<OctantLDB>())
                    .as_mut()
                    .unwrap_unchecked(),
                addr_of_mut!(*self.child_mut::<OctantRDB>())
                    .as_mut()
                    .unwrap_unchecked(),
                addr_of_mut!(*self.child_mut::<OctantLUB>())
                    .as_mut()
                    .unwrap_unchecked(),
                addr_of_mut!(*self.child_mut::<OctantRUB>())
                    .as_mut()
                    .unwrap_unchecked(),
            )
        }
    }
}

impl<T: Clone, Size: Unsigned, L: OctreeLayout, Depth: Unsigned, Index: Unsigned> Deref
    for OctreeNode<T, Size, L, Depth, Index>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

// TODO: Add a reference wrapper for Octree to allow reading data without copying
// it first.

/// Octree structure.
///
/// This structure is a smart wrapper of `Vec<T>` that provides safe octree
/// access semantics checked at compile time.
#[derive(Debug)]
#[repr(transparent)]
pub struct Octree<T: Clone, Depth: Unsigned, L: OctreeLayout = BreathFirst> {
    data: Vec<T>,
    _phantom: PhantomData<(Depth, L)>,
}

impl<T: Clone + Default, Depth: Unsigned, L: OctreeLayout> Default for Octree<T, Depth, L> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone, Depth: Unsigned, L: OctreeLayout> Octree<T, Depth, L> {
    /// Creates an octree with all nodes having the initial `value`.
    pub fn new(value: T) -> Self {
        let entry_count = subtree_length(Depth::USIZE);
        let mut result = Octree {
            data: Vec::with_capacity(entry_count),
            _phantom: PhantomData,
        };
        for i in 0..entry_count {
            result.data.push(value.clone());
        }
        result
    }

    /// Returns the byte size of the octree.
    pub const fn size() -> usize {
        crate::util::subtree_size::<T>(Depth::USIZE)
    }

    /// Returns the layout of the octree.
    pub fn layout() -> Layout {
        crate::util::subtree_layout::<T>(Depth::USIZE)
    }

    /// Returns a reference to the root node of the octree (first value).
    pub fn root(&self) -> &OctreeNode<T, Depth, L> {
        unsafe {
            (self.data.as_ptr() as *const OctreeNode<T, Depth, L>)
                .as_ref()
                .unwrap_unchecked()
        }
    }

    /// Returns a mutable reference to the root node of the octree (first value).
    pub fn root_mut(&mut self) -> &mut OctreeNode<T, Depth, L> {
        unsafe {
            (self.data.as_mut_ptr() as *mut OctreeNode<T, Depth, L>)
                .as_mut()
                .unwrap_unchecked()
        }
    }

    /// Fills the octree with the provided `value`.
    pub fn fill(&mut self, value: T) {
        self.data.clear();
        let count = subtree_length(Depth::USIZE);
        for i in 0..count {
            self.data.push(value.clone());
        }
    }

    /*
    /// Creates a new octree structure with root at the provided `position`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `position` is a valid pointer to
    /// the root octree node followed by data arranged in `L` layout.
    ///
    /// It's considered UB if data doesn't follow the layout described by `L`.
    ///
    /// It is allowed for the data to be uninitialized, but the caller must call
    /// [`Octree::fill`] before accessing or using any functions that rely on
    /// the octree being initialized.
    pub unsafe fn from_root_address(position: *mut T) -> Self {
        Octree {
            root: position as *mut OctreeNode<T, Depth, L>,
        }
    }
    */

    /// Returns a byte slice of data buffer.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                subtree_size::<T>(Depth::USIZE),
            )
        }
    }
}

impl<T: Clone, D: Unsigned> Octree<T, D, BreathFirst> {
    /// Returns a slice of `T` values at the given `depth`.
    pub fn layer_slice<Depth: Unsigned>(&self) -> &[T]
    where
        Depth: IsLessOrEqual<D>,
        LeEq<Depth, D>: Same<True>,
    {
        let skip = (0..Depth::USIZE)
            .map(|i| crate::util::layer_length(i))
            .sum();
        let len = crate::util::layer_length(Depth::USIZE);
        &self.data[skip..skip + len]
    }

    /// Returns a mutable slice of `T` values at the given `depth`.
    pub fn layer_slice_mut<Depth: Unsigned>(&mut self) -> &mut [T]
    where
        Depth: IsLessOrEqual<D>,
        LeEq<Depth, D>: Same<True>,
    {
        let skip = (0..Depth::USIZE)
            .map(|i| crate::util::layer_length(i))
            .sum();
        let len = crate::util::layer_length(Depth::USIZE);
        &mut self.data[skip..skip + len]
    }
}

impl<T: Clone, Depth: Unsigned, L: OctreeLayout> Deref for Octree<T, Depth, L> {
    type Target = OctreeNode<T, Depth, L>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.root() }
    }
}

impl<T: Clone, Depth: Unsigned, L: OctreeLayout> DerefMut for Octree<T, Depth, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.root_mut() }
    }
}

impl<T: Clone, Depth: Unsigned, L: OctreeLayout> AsRef<[T]> for Octree<T, Depth, L> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

/// Allows rearranging octree data between different layouts.
pub trait FromLayout<Other: OctreeLayout> {
    /// Constructs this octree from a an octree with a different memory
    /// different layout.
    fn from_layout<T: Clone, Depth: Unsigned>(other: Octree<T, Depth, Other>) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octree_index_bf_test() {
        let test = Octree::<usize, U3>::new(1);
        let root = test.data.as_ptr() as *const usize;

        unsafe {
            assert_eq!(
                addr_of!(*test.child::<OctantLDF>()) as *const usize,
                root.add(1)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantRDF>()) as *const usize,
                root.add(2)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantLUF>()) as *const usize,
                root.add(3)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantRUF>()) as *const usize,
                root.add(4)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantLDB>()) as *const usize,
                root.add(5)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantRDB>()) as *const usize,
                root.add(6)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantLUB>()) as *const usize,
                root.add(7)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantRUB>()) as *const usize,
                root.add(8)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantLDF>().child::<OctantLDF>()) as *const usize,
                root.add(9)
            );
            assert_eq!(
                addr_of!(*test.child::<OctantRDF>().child::<OctantLDF>()) as *const usize,
                root.add(17)
            );
            assert_eq!(
                addr_of!(*test
                    .child::<OctantLDF>()
                    .child::<OctantLDF>()
                    .child::<OctantLDF>()) as *const usize,
                root.add(73)
            );
        }
    }

    #[test]
    fn octree_get_set_bf_test() {
        let mut test = Octree::<usize, U2>::new(1);
        test.set_value(2);
        let ldf = test.child_mut::<OctantLDF>();
        ldf.set_value(3);
        test.child_mut::<OctantLUF>().set_value(4);
        test.child_mut::<OctantLUF>()
            .child_mut::<OctantRUF>()
            .set_value(5);
        test.child_mut::<OctantLUB>()
            .child_mut::<OctantRDB>()
            .set_value(6);

        assert_eq!(**test, 2);

        assert_eq!(**test.child::<OctantLDF>(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 3);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 3);

        assert_eq!(**test.child::<OctantRDF>(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantRDF>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantLUF>(), 4);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantLDF>().value(), 4);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantRDF>().value(), 4);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantLUF>().value(), 4);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantRUF>().value(), 5);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantLDB>().value(), 4);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantRDB>().value(), 4);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantLUB>().value(), 4);
        assert_eq!(*test.child::<OctantLUF>().child::<OctantRUB>().value(), 4);

        assert_eq!(**test.child::<OctantRUF>(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantRUF>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantLDB>(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLDB>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantRDB>(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantRDB>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantLUB>(), 2);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantRDB>().value(), 6);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLUB>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantRUB>(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantRUB>().child::<OctantRUB>().value(), 2);
    }

    #[test]
    fn octree_layout_bf_test() {
        let mut test = Octree::<usize, U2>::new(1);
        test.set_value(2);
        let ldf = test.child_mut::<OctantLDF>();
        ldf.set_value(3);
        test.child_mut::<OctantLUF>().set_value(4);
        test.child_mut::<OctantLUF>()
            .child_mut::<OctantRUF>()
            .set_value(5);
        test.child_mut::<OctantLUB>()
            .child_mut::<OctantRDB>()
            .set_value(6);

        let expected_data: [usize; 73] = [
            2, 3, 2, 4, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 5,
            4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 6, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ];

        let base_addr = unsafe { std::mem::transmute::<_, *const usize>(test) };

        for (i, value) in expected_data.into_iter().enumerate() {
            assert_eq!(unsafe { *(base_addr.add(i)) }, value);
        }
    }
}
