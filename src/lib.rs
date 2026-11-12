use std::array;
use std::iter;

mod buffer {
  pub trait Buffer<T> {
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
  }

  pub type Stack<T, const N: usize> = [T; N];

  impl<T, const N: usize> Buffer<T> for Stack<T, N> {
    fn as_slice(&self) -> &[T] {
      self.as_ref()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
      self.as_mut()
    }
  }

  pub type Heap<T, const N: usize> = Box<[T; N]>;

  impl<T, const N: usize> Buffer<T> for Heap<T, N> {
    fn as_slice(&self) -> &[T] {
      self.as_ref()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
      self.as_mut()
    }
  }

  pub type Borrowed<'a, T, const N: usize> = &'a mut [T; N];

  impl<'a, T, const N: usize> Buffer<T> for Borrowed<'a, T, N> {
    fn as_slice(&self) -> &[T] {
      self.as_ref()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
      self.as_mut()
    }
  }
}

mod private {
  pub trait Sealed {}
}

pub trait ModeTag: private::Sealed {
  type Buf<'a, T, const N: usize>: buffer::Buffer<T>
  where
    T: 'a;
}

pub enum Stack {}

impl private::Sealed for Stack {}

impl ModeTag for Stack {
  type Buf<'a, T, const N: usize>
    = buffer::Stack<T, N>
  where
    T: 'a;
}

pub enum Heap {}

impl private::Sealed for Heap {}

impl ModeTag for Heap {
  type Buf<'a, T, const N: usize>
    = buffer::Heap<T, N>
  where
    T: 'a;
}

pub enum Borrowed {}

impl private::Sealed for Borrowed {}

impl ModeTag for Borrowed {
  type Buf<'a, T, const N: usize>
    = buffer::Borrowed<'a, T, N>
  where
    T: 'a;
}

#[repr(transparent)]
pub struct Buffer<'a, M, T: 'a, const N: usize>
where
  M: ModeTag,
{
  inner: M::Buf<'a, T, N>,
}

impl<'a, T, const N: usize> Buffer<'a, Stack, T, N>
where
  T: Default + 'a,
{
  pub fn new() -> Self {
    Self {
      inner: array::from_fn(|_| Default::default()),
    }
  }
}

impl<'a, T, const N: usize> Default for Buffer<'a, Stack, T, N>
where
  T: Default + 'a,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, T, const N: usize> Buffer<'a, Heap, T, N>
where
  T: Default + 'a,
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

impl<'a, T, const N: usize> Default for Buffer<'a, Heap, T, N>
where
  T: Default + 'a,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, T, const N: usize> Buffer<'a, Borrowed, T, N>
where
  T: 'a,
{
  pub fn new(buf: &'a mut [T; N]) -> Self {
    Self { inner: buf }
  }
}

impl<'a, M: ModeTag, T, const N: usize> Buffer<'a, M, T, N>
where
  T: 'a,
{
  pub fn as_slice(&self) -> &[T] {
    buffer::Buffer::as_slice(&self.inner)
  }

  pub fn as_mut_slice(&mut self) -> &mut [T] {
    buffer::Buffer::as_mut_slice(&mut self.inner)
  }
}
