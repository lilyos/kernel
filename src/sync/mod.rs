mod singleton;
pub use singleton::Singleton;

mod mutex;
pub use mutex::{Mutex, MutexGuard};

mod rwlock;
pub use rwlock::RwLock;

mod semaphore;
pub use semaphore::Semaphore;

mod spinlock;
pub use spinlock::Spinlock;

/// Lie about something being sync
#[repr(transparent)]
pub struct FakeSync<T> {
    /// The inner data
    pub _inner: T,
}

impl<T> From<T> for FakeSync<T> {
    fn from(v: T) -> Self {
        Self { _inner: v }
    }
}

unsafe impl<T> Sync for FakeSync<T> {}
