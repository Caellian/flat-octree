use std::{
    alloc::Layout,
    mem::{align_of, size_of},
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr::{addr_of, addr_of_mut, null_mut},
};

#[derive(Debug)]
#[repr(C)]
pub struct OctreeNode<T: Clone, const SIZE: usize> {
    value: T,
}

impl<T: Clone, const DEPTH: usize> OctreeNode<T, DEPTH> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn set_value(&mut self, value: T) {
        let tailing = octree_size::<T>(DEPTH) / size_of::<T>();
        for i in 0..tailing {
            unsafe { (addr_of_mut!(*self) as *mut T).add(i).write(value.clone()) }
        }
    }
    pub fn child(&self, octant: Octant) -> &OctreeNode<T, { DEPTH - 1 }>
    where
        [(); DEPTH - 1]: Sized,
    {
        unsafe {
            let pos = addr_of!(*self) as *const u8;
            let pos =
                pos.add(octant.child_offset_df::<T>(DEPTH)) as *const OctreeNode<T, { DEPTH - 1 }>;
            pos.as_ref().unwrap_unchecked()
        }
    }
    pub fn child_mut(&mut self, octant: Octant) -> &mut OctreeNode<T, { DEPTH - 1 }>
    where
        [(); DEPTH - 1]: Sized,
    {
        unsafe {
            let pos = addr_of!(*self) as *mut u8;
            let pos =
                pos.add(octant.child_offset_df::<T>(DEPTH)) as *mut OctreeNode<T, { DEPTH - 1 }>;
            pos.as_mut().unwrap_unchecked()
        }
    }
    pub fn children<'a>(&'a self) -> impl Iterator<Item = &'a OctreeNode<T, { DEPTH - 1 }>> + 'a
    where
        [(); DEPTH - 1]: Sized,
    {
        Octant::ALL.iter().map(|it| self.child(*it))
    }
    pub fn children_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut OctreeNode<T, { DEPTH - 1 }>> + 'a
    where
        [(); DEPTH - 1]: Sized,
    {
        // SAFETY: while self if mutably borrowed, it's safe to access its
        // children as &mut
        Octant::ALL
            .iter()
            .map(|it| unsafe { &mut *addr_of_mut!(*self.child_mut(*it)) })
    }
}

impl<T: Clone, const DEPTH: usize> Index<Octant> for OctreeNode<T, DEPTH>
where
    [(); DEPTH - 1]: Sized,
{
    type Output = OctreeNode<T, { DEPTH - 1 }>;

    fn index(&self, octant: Octant) -> &Self::Output {
        self.child(octant)
    }
}

impl<T: Clone, const DEPTH: usize> IndexMut<Octant> for OctreeNode<T, DEPTH>
where
    [(); DEPTH - 1]: Sized,
{
    fn index_mut(&mut self, octant: Octant) -> &mut Self::Output {
        self.child_mut(octant)
    }
}

impl<T: Clone, const SIZE: usize> Deref for OctreeNode<T, SIZE> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Octree<T: Clone, const SIZE: usize> {
    root: *mut OctreeNode<T, SIZE>,
}

impl<T: Clone + Default, const SIZE: usize> Default for Octree<T, SIZE> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone, const SIZE: usize> Octree<T, SIZE> {
    pub(crate) fn uninit() -> Self {
        Octree { root: null_mut() }
    }

    pub fn fill(&mut self, value: T) {
        self.drop_root();
        let result: *mut OctreeNode<T, SIZE> =
            unsafe { std::alloc::alloc(octree_layout::<T>(SIZE)) as *mut OctreeNode<T, SIZE> };

        let count = octree_size::<T>(SIZE) / size_of::<T>();
        for i in 0..count {
            unsafe { (result as *mut T).add(i).write(value.clone()) }
        }

        self.root = result as *mut OctreeNode<T, SIZE>;
    }

    pub fn new(value: T) -> Self {
        let mut result = Self::uninit();
        result.fill(value);
        result
    }

    fn drop_root(&mut self) {
        unsafe {
            if let Some(root) = self.root.as_mut() {
                let root = addr_of_mut!(*root) as *mut u8;
                std::alloc::dealloc(root, octree_layout::<T>(SIZE));
                self.root = null_mut();
            }
        }
    }
}

impl<T: Clone, const SIZE: usize> Drop for Octree<T, SIZE> {
    fn drop(&mut self) {
        self.drop_root()
    }
}

impl<T: Clone, const SIZE: usize> Deref for Octree<T, SIZE> {
    type Target = OctreeNode<T, SIZE>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.root.as_ref().expect("octree not initialized") }
    }
}

impl<T: Clone, const SIZE: usize> DerefMut for Octree<T, SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.root.as_mut().expect("octree not initialized") }
    }
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

    #[test]
    fn octree_get_set_test() {
        let mut test = Octree::<usize, 2>::new(1);
        test.set_value(2);
        let fbl: &mut OctreeNode<usize, 1> = &mut test[Octant::FBL];
        fbl.set_value(3);
        test[Octant::RTR].set_value(4);
        test[Octant::RBL][Octant::RBL].set_value(5);

        assert_eq!(**test, 2);

        assert_eq!(*test[Octant::FTL], 2);
        assert_eq!(*test[Octant::FTL][Octant::FTL], 2);
        assert_eq!(*test[Octant::FTL][Octant::FTR], 2);
        assert_eq!(*test[Octant::FTL][Octant::FBR], 2);
        assert_eq!(*test[Octant::FTL][Octant::FBL], 2);
        assert_eq!(*test[Octant::FTL][Octant::RTL], 2);
        assert_eq!(*test[Octant::FTL][Octant::RTR], 2);
        assert_eq!(*test[Octant::FTL][Octant::RBR], 2);
        assert_eq!(*test[Octant::FTL][Octant::RBL], 2);

        assert_eq!(*test[Octant::FTR], 2);
        assert_eq!(*test[Octant::FTR][Octant::FTL], 2);
        assert_eq!(*test[Octant::FTR][Octant::FTR], 2);
        assert_eq!(*test[Octant::FTR][Octant::FBR], 2);
        assert_eq!(*test[Octant::FTR][Octant::FBL], 2);
        assert_eq!(*test[Octant::FTR][Octant::RTL], 2);
        assert_eq!(*test[Octant::FTR][Octant::RTR], 2);
        assert_eq!(*test[Octant::FTR][Octant::RBR], 2);
        assert_eq!(*test[Octant::FTR][Octant::RBL], 2);

        assert_eq!(*test[Octant::FBR], 2);
        assert_eq!(*test[Octant::FBR][Octant::FTL], 2);
        assert_eq!(*test[Octant::FBR][Octant::FTR], 2);
        assert_eq!(*test[Octant::FBR][Octant::FBR], 2);
        assert_eq!(*test[Octant::FBR][Octant::FBL], 2);
        assert_eq!(*test[Octant::FBR][Octant::RTL], 2);
        assert_eq!(*test[Octant::FBR][Octant::RTR], 2);
        assert_eq!(*test[Octant::FBR][Octant::RBR], 2);
        assert_eq!(*test[Octant::FBR][Octant::RBL], 2);

        assert_eq!(*test[Octant::FBL], 3);
        assert_eq!(*test[Octant::FBL][Octant::FTL], 3);
        assert_eq!(*test[Octant::FBL][Octant::FTR], 3);
        assert_eq!(*test[Octant::FBL][Octant::FBR], 3);
        assert_eq!(*test[Octant::FBL][Octant::FBL], 3);
        assert_eq!(*test[Octant::FBL][Octant::RTL], 3);
        assert_eq!(*test[Octant::FBL][Octant::RTR], 3);
        assert_eq!(*test[Octant::FBL][Octant::RBR], 3);
        assert_eq!(*test[Octant::FBL][Octant::RBL], 3);

        assert_eq!(*test[Octant::RTL], 2);
        assert_eq!(*test[Octant::RTL][Octant::FTL], 2);
        assert_eq!(*test[Octant::RTL][Octant::FTR], 2);
        assert_eq!(*test[Octant::RTL][Octant::FBR], 2);
        assert_eq!(*test[Octant::RTL][Octant::FBL], 2);
        assert_eq!(*test[Octant::RTL][Octant::RTL], 2);
        assert_eq!(*test[Octant::RTL][Octant::RTR], 2);
        assert_eq!(*test[Octant::RTL][Octant::RBR], 2);
        assert_eq!(*test[Octant::RTL][Octant::RBL], 2);

        assert_eq!(*test[Octant::RTR], 4);
        assert_eq!(*test[Octant::RTR][Octant::FTL], 4);
        assert_eq!(*test[Octant::RTR][Octant::FTR], 4);
        assert_eq!(*test[Octant::RTR][Octant::FBR], 4);
        assert_eq!(*test[Octant::RTR][Octant::FBL], 4);
        assert_eq!(*test[Octant::RTR][Octant::RTL], 4);
        assert_eq!(*test[Octant::RTR][Octant::RTR], 4);
        assert_eq!(*test[Octant::RTR][Octant::RBR], 4);
        assert_eq!(*test[Octant::RTR][Octant::RBL], 4);

        assert_eq!(*test[Octant::RBR], 2);
        assert_eq!(*test[Octant::RBR][Octant::FTL], 2);
        assert_eq!(*test[Octant::RBR][Octant::FTR], 2);
        assert_eq!(*test[Octant::RBR][Octant::FBR], 2);
        assert_eq!(*test[Octant::RBR][Octant::FBL], 2);
        assert_eq!(*test[Octant::RBR][Octant::RTL], 2);
        assert_eq!(*test[Octant::RBR][Octant::RTR], 2);
        assert_eq!(*test[Octant::RBR][Octant::RBR], 2);
        assert_eq!(*test[Octant::RBR][Octant::RBL], 2);

        assert_eq!(*test[Octant::RBL], 2);
        assert_eq!(*test[Octant::RBL][Octant::FTL], 2);
        assert_eq!(*test[Octant::RBL][Octant::FTR], 2);
        assert_eq!(*test[Octant::RBL][Octant::FBR], 2);
        assert_eq!(*test[Octant::RBL][Octant::FBL], 2);
        assert_eq!(*test[Octant::RBL][Octant::RTL], 2);
        assert_eq!(*test[Octant::RBL][Octant::RTR], 2);
        assert_eq!(*test[Octant::RBL][Octant::RBR], 2);
        assert_eq!(*test[Octant::RBL][Octant::RBL], 5);
    }

    #[test]
    fn octree_layout_test() {
        let mut test = Octree::<usize, 2>::new(1);
        test.set_value(2);
        let fbl: &mut OctreeNode<usize, 1> = &mut test[Octant::FBL];
        fbl.set_value(3);
        test[Octant::RTR].set_value(4);
        test[Octant::RBL][Octant::RBL].set_value(5);

        let expected_data: [usize; 73] = [
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 5,
        ];

        let inner = unsafe { std::mem::transmute::<_, *mut OctreeNode<usize, 2>>(test) };
        let base_addr = inner as *const usize;

        for (i, value) in expected_data.into_iter().enumerate() {
            assert_eq!(unsafe { *(base_addr.add(i)) }, value);
        }
    }
}
