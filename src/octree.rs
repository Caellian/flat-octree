use std::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
    ops::{Add, Deref, DerefMut, Index, IndexMut, Mul, Sub},
    process::Output,
    ptr::{addr_of, addr_of_mut, null_mut},
};

use typenum::*;

use crate::{
    layout::{BreathFirst, MemoryLayout},
    octant::*,
    subtree_size,
};

mod sealed {
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

type ChildIndex<I, ChildOctant> =
    <<I as Mul<U8>>::Output as Add<<ChildOctant as OctantT>::IndexT>>::Output;

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

#[derive(Debug)]
#[repr(transparent)]
pub struct OctreeNode<
    T: Clone,
    Size: Unsigned,
    L: MemoryLayout,
    Depth: Unsigned = Size,
    LayerIndex: Unsigned = U0,
> {
    value: T,
    _phantom: PhantomData<(L, Size, Depth, LayerIndex)>,
}

impl<T: Clone, S: Unsigned, L: MemoryLayout, D: Unsigned, I: Unsigned> OctreeNode<T, S, L, D, I> {
    pub const fn octant(&self) -> Octant
    where
        D: IsLess<S>,
        Le<D, S>: Same<True>,
    {
        Octant::ALL[I::USIZE % 8]
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn set_value(&mut self, value: T) {
        L::fill(
            addr_of_mut!(self.value),
            value,
            S::USIZE,
            D::USIZE,
            I::USIZE,
        )
    }

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
        <<I as Mul<U8>>::Output as Add<U0>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U1>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U2>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U3>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U4>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U5>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U6>>::Output: Unsigned,
        <<I as Mul<U8>>::Output as Add<U7>>::Output: Unsigned,
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

impl<T: Clone, Size: Unsigned, L: MemoryLayout, Depth: Unsigned, Index: Unsigned> Deref
    for OctreeNode<T, Size, L, Depth, Index>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Octree<T: Clone, Depth: Unsigned, L: MemoryLayout = BreathFirst> {
    root: *mut OctreeNode<T, Depth, L>,
}

impl<T: Clone + Default, Depth: Unsigned, L: MemoryLayout> Default for Octree<T, Depth, L> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone, Depth: Unsigned, L: MemoryLayout> Octree<T, Depth, L> {
    pub const fn size() -> usize {
        crate::subtree_size::<T>(Depth::USIZE)
    }

    pub fn layout() -> Layout {
        crate::subtree_layout::<T>(Depth::USIZE)
    }

    pub(crate) fn uninit() -> Self {
        Octree { root: null_mut() }
    }

    pub fn fill(&mut self, value: T) {
        self.drop_root();
        let result: *mut OctreeNode<T, Depth, L> =
            unsafe { std::alloc::alloc(Self::layout()) as *mut OctreeNode<T, Depth, L> };

        let count = Self::size() / size_of::<T>();
        for i in 0..count {
            unsafe { (result as *mut T).add(i).write(value.clone()) }
        }

        self.root = result as *mut OctreeNode<T, Depth, L>;
    }

    pub fn new(value: T) -> Self {
        let mut result = Self::uninit();
        result.fill(value);
        result
    }

    pub unsafe fn from_root_address(position: *mut T) -> Self {
        Octree {
            root: position as *mut OctreeNode<T, Depth, L>,
        }
    }

    fn drop_root(&mut self) {
        unsafe {
            if let Some(root) = self.root.as_mut() {
                let root = addr_of_mut!(*root) as *mut u8;
                std::alloc::dealloc(root, Self::layout());
                self.root = null_mut();
            }
        }
    }
}

impl<T: Clone, Depth: Unsigned, L: MemoryLayout> Drop for Octree<T, Depth, L> {
    fn drop(&mut self) {
        self.drop_root()
    }
}

impl<T: Clone, Depth: Unsigned, L: MemoryLayout> Deref for Octree<T, Depth, L> {
    type Target = OctreeNode<T, Depth, L>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.root.as_ref().expect("octree not initialized") }
    }
}

impl<T: Clone, Depth: Unsigned, L: MemoryLayout> DerefMut for Octree<T, Depth, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.root.as_mut().expect("octree not initialized") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octree_get_set_test() {
        let mut test = Octree::<usize, U2>::new(1);
        test.set_value(2);
        let fbl = test.child_mut::<OctantLDF>();
        fbl.set_value(3);
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
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantLUF>(), 4);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 4);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 4);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 4);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 5);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 4);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 4);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 4);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 4);

        assert_eq!(**test.child::<OctantRUF>(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantLDB>(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantRDB>(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantLUB>(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 5);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 2);

        assert_eq!(**test.child::<OctantRUB>(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUF>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRDB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantLUB>().value(), 2);
        assert_eq!(*test.child::<OctantLDF>().child::<OctantRUB>().value(), 2);
    }

    /*
    #[test]
    fn octree_layout_test() {
        let mut test = Octree::<usize, U2>::new(1);
        test.set_value(2);
        let fbl: &mut OctreeNode<usize, U2, U1> = &mut test.child_mut::<OctantLDF>();
        fbl.set_value(3);
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
            2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ];

        let inner = unsafe { std::mem::transmute::<_, *mut OctreeNode<usize, U2>>(test) };
        let base_addr = inner as *const usize;

        for (i, value) in expected_data.into_iter().enumerate() {
            assert_eq!(unsafe { *(base_addr.add(i)) }, value);
        }
    } */
}
