use crate::{Output, State, Symphonia, VOLUME_REDUCTION};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

static PLAYER_STATE: OnceLock<Arc<PlayerState>> = OnceLock::new();

pub struct Mailbox<T> {
    pub slot: UnsafeCell<Option<T>>,
    pub full: AtomicBool,
}

impl<T> Mailbox<T> {
    pub const fn new() -> Self {
        Self {
            slot: UnsafeCell::new(None),
            full: AtomicBool::new(false),
        }
    }

    pub fn publish(&self, value: T) {
        unsafe {
            *self.slot.get() = Some(value);
        }
        self.full.store(true, Ordering::Release);
    }

    pub fn take(&self) -> Option<T> {
        if !self.full.swap(false, Ordering::AcqRel) {
            return None;
        }
        unsafe { (*self.slot.get()).take() }
    }
}

unsafe impl<T: Send> Send for Mailbox<T> {}
unsafe impl<T: Send> Sync for Mailbox<T> {}

pub struct PlayerState {
    pub state: AtomicU8,
    pub volume: AtomicU32,
    pub gain: AtomicU32,
    pub elapsed: AtomicU64,
    pub duration: AtomicU64,
    pub seek: AtomicU64,
    pub finished: AtomicBool,
    pub decoder_pending: AtomicBool,
    pub pending_decoder: Mailbox<Symphonia>,
    pub pending_output: Mailbox<Output>,
}

impl PlayerState {
    pub fn global() -> Arc<PlayerState> {
        PLAYER_STATE
            .get_or_init(|| {
                Arc::new(PlayerState {
                    state: AtomicU8::new(State::Stopped as u8),
                    volume: AtomicU32::new(((15.0 / VOLUME_REDUCTION) * 0.5).to_bits()),
                    gain: AtomicU32::new(0.5f32.to_bits()),
                    elapsed: AtomicU64::new(0),
                    duration: AtomicU64::new(0),
                    seek: AtomicU64::new(u64::MAX),
                    finished: AtomicBool::new(false),
                    decoder_pending: AtomicBool::new(false),
                    pending_decoder: Mailbox::new(),
                    pending_output: Mailbox::new(),
                })
            })
            .clone()
    }
}

unsafe impl Send for PlayerState {}
unsafe impl Sync for PlayerState {}
