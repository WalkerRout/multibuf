use std::array;
use std::iter;

mod buffer {
  pub trait Buffer<T> {
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
  }

  pub struct StackBuf<T, const N: usize> {
    pub buf: [T; N],
  }

  impl<T, const N: usize> Buffer<T> for StackBuf<T, N> {
    fn as_slice(&self) -> &[T] {
      &self.buf
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
      &mut self.buf
    }
  }

  pub struct HeapBuf<T, const N: usize> {
    pub buf: Box<[T; N]>,
  }

  impl<T, const N: usize> Buffer<T> for HeapBuf<T, N> {
    fn as_slice(&self) -> &[T] {
      self.buf.as_slice()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
      self.buf.as_mut_slice()
    }
  }

  pub struct ExternalBuf<'a, T, const N: usize> {
    pub buf: &'a mut [T; N],
  }

  impl<'a, T, const N: usize> Buffer<T> for ExternalBuf<'a, T, N> {
    fn as_slice(&self) -> &[T] {
      self.buf
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
      self.buf
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
    = buffer::StackBuf<T, N>
  where
    T: 'a;
}

pub enum Heap {}

impl private::Sealed for Heap {}

impl ModeTag for Heap {
  type Buf<'a, T, const N: usize>
    = buffer::HeapBuf<T, N>
  where
    T: 'a;
}

pub enum External {}

impl private::Sealed for External {}

impl ModeTag for External {
  type Buf<'a, T, const N: usize>
    = buffer::ExternalBuf<'a, T, N>
  where
    T: 'a;
}

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
      inner: buffer::StackBuf {
        buf: array::from_fn(|_| Default::default()),
      },
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
      inner: buffer::HeapBuf {
        buf: iter::repeat_with(Default::default)
          .take(N)
          .collect::<Box<[T]>>()
          .try_into()
          // safety; we take exactly N, just hoping compiler elides len check
          .unwrap_or_else(|_| unreachable!()),
      },
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

impl<'a, T, const N: usize> Buffer<'a, External, T, N>
where
  T: 'a,
{
  pub fn new(buf: &'a mut [T; N]) -> Self {
    Self {
      inner: buffer::ExternalBuf { buf },
    }
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
