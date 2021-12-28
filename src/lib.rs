mod mutref;
use mutref::get_mutable_ref;
use core::sync::atomic::{AtomicUsize, Ordering};

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

    fn reserve(&'a self, sz: usize) -> usize {
        if sz == 0 {
            return 0;
        }
        let mut old = self.next.load(Ordering::Relaxed);
        loop {
            let new = old + sz;
            if new >= self.bytes.len() {
                return 0;
            }
            match self
                .next
                .compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed)
            {
                Ok(_) => return new,
                Err(new) => old = new,
            }
        }
    }

    pub fn alloc(&'a self, sz: usize) -> Option<&'a [u8]> {
        let new = self.reserve(sz);
        if new == 0 {
            None
        } else {
            let old = new - sz;
            Some(&self.bytes[old..new])
        }
    }
    
    pub fn alloc_mut(&'a self, sz: usize) -> Option<&'a mut [u8]> {
        let new = self.reserve(sz);
        if new == 0 {
            None
        } else {
            let self_mut = unsafe { get_mutable_ref(self) };
            let old = new - sz;
            Some(&mut self_mut.bytes[old..new])
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let mut bytes: [u8; 25] = [0; 25];
        for n in 0..bytes.len() {
            bytes[n] = (n % 255) as u8;
        }

        let memory = crate::Buffer::init_from_bytes(&mut bytes);
        assert_eq!(memory.len(), 25);

        let maybe_buf0 = memory.alloc(12);
        assert_ne!(maybe_buf0, None);
        let buf0 = maybe_buf0.unwrap();
        assert_eq!(buf0.len(), 12);
        assert_eq!(buf0[0], 0);
        assert_eq!(buf0[5], 5);

        let maybe_buf1 = memory.alloc_mut(10);
        assert_ne!(maybe_buf1, None);
        let buf1 = maybe_buf1.unwrap();
        assert_eq!(buf1.len(), 10);
        assert_eq!(buf1[0], 12);
        assert_eq!(buf1[1], 13);
        buf1[0] = 255;
        assert_eq!(buf1[0], 255);

        assert_eq!(memory.remaining(), 3);

        let bytes3 = memory.alloc(20);
        assert_eq!(None, bytes3);
        assert_eq!(memory.remaining(), 3);
    }
}
