use crate::mutref::get_mutable_ref;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Buffer<'a> {
    bytes: &'a mut [u8],
    next: AtomicUsize,
}

impl<'a> Buffer<'a> {
    pub fn init_from_bytes(bytes: &'a mut [u8]) -> Self {
        Self {
            bytes,
            next: AtomicUsize::new(0),
        }
    }

    pub fn len(self: &Self) -> usize {
        return self.bytes.len();
    }

    pub fn alloc(self: &'a Self, sz: usize) -> Option<&'a mut [u8]> {
        let begin = self.next.fetch_add(sz, Ordering::SeqCst);
        let end = begin + sz;
        if end >= self.bytes.len() {
            return None;
        }
        let self_mut = unsafe { get_mutable_ref(self) };
        Some(&mut self_mut.bytes[begin..end])
    }
}
