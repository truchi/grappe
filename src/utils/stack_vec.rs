use std::slice::SliceIndex;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StackVec<T, const N: usize> {
    len:   u16,
    items: [T; N],
}

impl<T: Default, const N: usize> Default for StackVec<T, N> {
    fn default() -> Self {
        Self {
            len:   0,
            items: [(); N].map(|_| Default::default()),
        }
    }
}

impl<T, const N: usize> AsRef<[T]> for StackVec<T, N> {
    fn as_ref(&self) -> &[T] {
        unsafe { get!(&self.items, ..self.len()) }
    }
}

impl<T, const N: usize> AsMut<[T]> for StackVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { get!(mut &mut self.items, ..self.len as usize) }
    }
}

impl<T, const N: usize> StackVec<T, N> {
    pub fn from_slice(mut slice: &[T]) -> impl '_ + Iterator<Item = Self>
    where
        T: Copy + Default,
    {
        std::iter::from_fn(move || {
            if slice.len() == 0 {
                None
            } else {
                let (before, after) = unsafe { split(slice, N) };
                slice = after;

                debug_assert!(before.len() <= N);
                debug_assert!(u16::try_from(before.len()).is_ok());

                let len = before.len() as u16;
                let mut items = [(); N].map(|_| Default::default());

                unsafe { copy(before, get!(mut &mut items, ..before.len())) };

                Some(Self { len, items })
            }
        })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn get<I: SliceIndex<[T]>>(&self, index: I) -> Option<&I::Output> {
        self.as_ref().get(index)
    }

    pub fn get_mut<I: SliceIndex<[T]>>(&mut self, index: I) -> Option<&mut I::Output> {
        self.as_mut().get_mut(index)
    }

    pub unsafe fn get_unchecked<I: SliceIndex<[T]>>(&self, index: I) -> &I::Output {
        self.as_ref().get_unchecked(index)
    }

    pub unsafe fn get_unchecked_mut<I: SliceIndex<[T]>>(&mut self, index: I) -> &mut I::Output {
        self.as_mut().get_unchecked_mut(index)
    }

    pub fn extend<'a>(&mut self, slice: &'a [T]) -> impl 'a + Iterator<Item = Self>
    where
        T: Copy + Default,
    {
        let (before, after) = split(slice, N - self.len());

        let range = self.len()..self.len() + before.len();
        unsafe { copy(before, get!(mut &mut self.items, range)) };

        Self::from_slice(after)
    }
}

fn split<T>(slice: &[T], at: usize) -> (&[T], &[T]) {
    if at >= slice.len() {
        (slice, &[])
    } else {
        unsafe { (get!(slice, ..at), get!(slice, at..)) }
    }
}

unsafe fn copy<T: Copy>(src: &[T], dest: &mut [T]) {
    debug_assert!(src.len() == dest.len());

    std::ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), dest.len());
}
