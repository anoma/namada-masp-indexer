use std::sync::atomic::{self, AtomicBool};

static EXIT_HANDLE: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn must_exit() -> bool {
    EXIT_HANDLE.load(atomic::Ordering::Relaxed)
}

#[inline]
pub fn exit() {
    EXIT_HANDLE.store(true, atomic::Ordering::Relaxed);
}
