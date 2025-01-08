use std::alloc::{alloc, dealloc, Layout};

/// A trait representing a buffer that is suitably sized and aligned.
///
/// Provides an abstraction over different types of buffers with alignment
/// guarantees. Implementors must ensure that all returned pointers are
/// correctly aligned for the specified alignment.
///
/// # Safety
///
/// Implementations may use `unsafe` code for memory allocation and pointer
/// arithmetic. Such `unsafe` usage must guarantee:
/// - Proper alignment as specified.
/// - No memory leaks or undefined behavior.
pub(crate) trait AlignedBuffer: Sized {
    /// Creates a new buffer of `size` bytes with the given `alignment`.
    ///
    /// # Parameters
    ///
    /// - `size`: The size of the buffer in bytes.
    /// - `alignment`: The required alignment for the buffer.
    ///
    /// # Returns
    ///
    /// A `Result` containing the buffer if successful, or an error otherwise.
    fn new(size: usize, alignment: usize) -> Result<Self, Error>;

    /// Returns a raw immutable pointer to the underlying buffer.
    fn as_ptr(&self) -> *const u8;

    /// Returns a raw mutable pointer to the underlying buffer.
    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Returns an immutable slice of the underlying bytes.
    fn as_slice(&self) -> &[u8];

    /// Returns a mutable slice of the underlying bytes.
    fn as_mut_slice(&mut self) -> &mut [u8];
}

/// Errors that can occur when creating an `AlignedBuffer`.
#[derive(Debug)]
pub(crate) enum Error {
    /// The requested buffer size exceeds the maximum capacity.
    SizeTooLarge { requested: usize, max: usize },
    /// The buffer pointer is not aligned properly to the requested alignment.
    BadAlignment { alignment: usize },
}

/// A heap-allocated buffer that ensures proper size and alignment.
///
/// This implementation uses `std::alloc` for dynamic memory allocation with a
/// specified size and alignment. Memory is deallocated when the buffer goes out
/// of scope.
///
/// # Safety
///
/// Unsafe operations are used in:
/// - Allocation: Ensures memory is allocated using the requested `Layout`.
/// - Deallocation: Ensures memory is properly freed when the buffer is dropped.
///
/// All `unsafe` code within this struct guarantees alignment and size as
/// specified by the user, as well as memory safety.
pub(crate) struct HeapBuffer {
    ptr: *mut u8,
    layout: Layout,
    size: usize,
}

impl AlignedBuffer for HeapBuffer {
    fn new(size: usize, alignment: usize) -> Result<Self, Error> {
        if !alignment.is_power_of_two() {
            return Err(Error::BadAlignment { alignment });
        }

        let layout = Layout::from_size_align(size, alignment)
            .map_err(|_| Error::BadAlignment { alignment })?;

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(Error::SizeTooLarge {
                requested: size,
                max: usize::MAX,
            });
        }

        Ok(Self { ptr, layout, size })
    }

    fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
    }
}

impl Drop for HeapBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.layout) };
    }
}

/// A stack-allocated buffer with dynamic alignment validation.
///
/// This buffer is a fixed-size array allocated on the stack, with runtime
/// checks for alignment and size. Misaligned buffers currently result in an
/// error, but future implementations might pad the buffer to ensure alignment.
///
/// # Notes
///
/// - The size of the buffer is specified using const generics (`N`) at compile
///   time, allowing users to define the buffer size according to their needs.
/// - The buffer's size is statically allocated and cannot change at runtime.
/// - Alignment checks ensure that the buffer meets the specified requirements,
///   but padding for misalignment is not yet supported.
pub(crate) struct StackBuffer<const N: usize> {
    data: [u8; N],
    size: usize,
}

impl<const N: usize> AlignedBuffer for StackBuffer<N> {
    fn new(size: usize, alignment: usize) -> Result<Self, Error> {
        if size > N {
            return Err(Error::SizeTooLarge {
                requested: size,
                max: N,
            });
        }
        if !alignment.is_power_of_two() {
            return Err(Error::BadAlignment { alignment });
        }

        let buffer = Self {
            data: [0u8; N],
            size,
        };

        if buffer.data.as_ptr().align_offset(alignment) != 0 {
            return Err(Error::BadAlignment { alignment });
        }

        Ok(buffer)
    }

    fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    fn as_slice(&self) -> &[u8] {
        &self.data[..self.size]
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.size]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_buffer_ok() {
        let mut buffer = HeapBuffer::new(128, 64).expect("Failed to create HeapBuffer");
        assert_eq!(buffer.as_slice().len(), 128);
        assert_eq!(buffer.as_mut_slice().as_ptr() as usize % 64, 0);

        let ptr = buffer.as_ptr();
        let mut_ptr = buffer.as_mut_ptr();
        unsafe {
            *mut_ptr = 42;
            assert_eq!(*ptr, 42);
        }
    }

    #[test]
    fn heap_buffer_invalid_alignment() {
        let result = HeapBuffer::new(128, 3);
        assert!(matches!(result, Err(Error::BadAlignment { .. })));
    }

    #[test]
    fn stack_buffer_ok() {
        let mut buffer = StackBuffer::<256>::new(128, 16).expect("Failed to create StackBuffer");
        assert_eq!(buffer.as_slice().len(), 128);

        let ptr = buffer.as_ptr();
        let mut_ptr = buffer.as_mut_ptr();
        unsafe {
            *mut_ptr = 42;
            assert_eq!(*ptr, 42);
        }
    }

    #[test]
    fn stack_buffer_invalid_alignment() {
        let result = StackBuffer::<256>::new(128, 7);
        assert!(matches!(result, Err(Error::BadAlignment { .. })));
    }

    #[test]
    fn stack_buffer_too_large() {
        let result = StackBuffer::<128>::new(256, 8);
        assert!(matches!(
            result,
            Err(Error::SizeTooLarge {
                requested: 256,
                max: 128
            })
        ));
    }
}
