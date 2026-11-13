use std::array;
use std::convert::Infallible;
use std::iter;
use std::marker::PhantomData;

mod buffer {
  use std::borrow::BorrowMut;

  pub type Stack<T, const N: usize> = [T; N];

  pub type Heap<T, const N: usize> = Box<[T; N]>;

  pub type Borrowed<'a, T, const N: usize> = &'a mut [T; N];

  pub trait Buffer<T, const N: usize>: BorrowMut<[T; N]> {
    fn as_slice(&self) -> &[T] {
      self.borrow()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
      self.borrow_mut()
    }
  }

  impl<T, B, const N: usize> Buffer<T, N> for B where B: BorrowMut<[T; N]> {}
}

mod private {
  pub trait Sealed {}
}

pub trait ModeTag<T, const N: usize>: private::Sealed {
  type Buf: buffer::Buffer<T, N>;
}

pub enum Stack {}

impl private::Sealed for Stack {}

impl<T, const N: usize> ModeTag<T, N> for Stack {
  type Buf = buffer::Stack<T, N>;
}

pub enum Heap {}

impl private::Sealed for Heap {}

impl<T, const N: usize> ModeTag<T, N> for Heap {
  type Buf = buffer::Heap<T, N>;
}

// could use &'a ! but that still costs 8 bytes...
pub struct Borrowed<'a>(Infallible, PhantomData<&'a ()>);

impl<'a> private::Sealed for Borrowed<'a> {}

impl<'a, T, const N: usize> ModeTag<T, N> for Borrowed<'a>
where
  T: 'a,
{
  type Buf = buffer::Borrowed<'a, T, N>;
}

#[repr(transparent)]
pub struct Buffer<M, T, const N: usize>
where
  M: ModeTag<T, N>,
{
  inner: M::Buf,
}

impl<T, const N: usize> Buffer<Stack, T, N>
where
  T: Default,
{
  pub fn new() -> Self {
    Self {
      inner: array::from_fn(|_| Default::default()),
    }
  }
}

impl<T, const N: usize> Default for Buffer<Stack, T, N>
where
  T: Default,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const N: usize> Buffer<Heap, T, N>
where
  T: Default,
{
  pub fn new() -> Self {
    Self {
      inner: iter::repeat_with(Default::default)
        .take(N)
        .collect::<Box<[T]>>()
        .try_into()
        // safety; we take exactly N, just hoping compiler elides len check
        .unwrap_or_else(|_| unreachable!()),
    }
  }
}

impl<T, const N: usize> Default for Buffer<Heap, T, N>
where
  T: Default,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, T, const N: usize> Buffer<Borrowed<'a>, T, N>
where
  T: 'a,
{
  pub fn new(buf: &'a mut [T; N]) -> Self {
    Self { inner: buf }
  }
}

impl<M, T, const N: usize> Buffer<M, T, N>
where
  M: ModeTag<T, N>,
{
  pub fn as_slice(&self) -> &[T] {
    buffer::Buffer::as_slice(&self.inner)
  }

  pub fn as_mut_slice(&mut self) -> &mut [T] {
    buffer::Buffer::as_mut_slice(&mut self.inner)
  }
}
