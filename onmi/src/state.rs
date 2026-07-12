use crate::{Output, State, Symphonia};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

pub const DEFAULT_VOLUME_REDUCTION: f32 = 75.0;

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RuntimeError {
    None = 0,
    OutputOpen = 1,
    StreamStart = 2,
}

pub struct Mailbox<T> {
    pub ptr: AtomicPtr<T>,
}

impl<T> Mailbox<T> {
    pub const fn new() -> Self {
        Self {
            ptr: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn publish(&self, value: T) {
        let new = Box::into_raw(Box::new(value));
        let old = self.ptr.swap(new, Ordering::AcqRel);
        if !old.is_null() {
            unsafe {
                drop(Box::from_raw(old));
            }
        }
    }

    pub fn take(&self) -> Option<T> {
        let p = self.ptr.swap(ptr::null_mut(), Ordering::AcqRel);
        if p.is_null() {
            None
        } else {
            unsafe { Some(*Box::from_raw(p)) }
        }
    }
}

impl<T> Drop for Mailbox<T> {
    fn drop(&mut self) {
        let p = *self.ptr.get_mut();
        if !p.is_null() {
            unsafe {
                drop(Box::from_raw(p));
            }
        }
    }
}

unsafe impl<T: Send> Send for Mailbox<T> {}
unsafe impl<T: Send> Sync for Mailbox<T> {}

pub struct PlayerState {
    pub state: AtomicU8,
    pub volume: AtomicU32,
    pub gain: AtomicU32,
    pub volume_reduction: AtomicU32,
    pub elapsed: AtomicU64,
    pub duration: AtomicU64,
    pub seek: AtomicU64,
    pub finished: AtomicBool,
    pub decoder_pending: AtomicBool,
    pub shutdown: AtomicBool,
    pub follow_default: AtomicBool,
    pub last_error: AtomicU8,
    pub pending_decoder: Mailbox<Symphonia>,
    pub pending_output: Mailbox<Output>,
}

impl PlayerState {
    pub fn new() -> Arc<Self> {
        Arc::new(PlayerState {
            state: AtomicU8::new(State::Stopped as u8),
            volume: AtomicU32::new(((15.0 / DEFAULT_VOLUME_REDUCTION) * 0.5).to_bits()),
            gain: AtomicU32::new(0.5f32.to_bits()),
            volume_reduction: AtomicU32::new(DEFAULT_VOLUME_REDUCTION.to_bits()),
            elapsed: AtomicU64::new(0),
            duration: AtomicU64::new(0),
            seek: AtomicU64::new(u64::MAX),
            finished: AtomicBool::new(false),
            decoder_pending: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
            follow_default: AtomicBool::new(false),
            last_error: AtomicU8::new(RuntimeError::None as u8),
            pending_decoder: Mailbox::new(),
            pending_output: Mailbox::new(),
        })
    }

    pub fn set_error(&self, error: RuntimeError) {
        self.last_error.store(error as u8, Ordering::Relaxed);
    }

    pub fn mark_finished(&self) {
        self.finished.store(true, Ordering::Relaxed);
        self.state.store(State::Stopped as u8, Ordering::Relaxed);
    }
}

unsafe impl Send for PlayerState {}
unsafe impl Sync for PlayerState {}
