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

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.next.load(Ordering::Relaxed)
    }

    pub fn alloc(self: &'a Self, sz: usize) -> Option<&'a mut [u8]> {
        let mut old = self.next.load(Ordering::Relaxed);
        loop {
            let new = old + sz;
            if new  >= self.bytes.len() {
                return None;
            }
            match self.next.compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(new) => old = new,
            }
        }
        let self_mut = unsafe { get_mutable_ref(self) };
        Some(&mut self_mut.bytes[old..old+sz])
    }
}
