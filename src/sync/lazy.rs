use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Lazy<T> {
    init: AtomicBool,
    func: fn() -> T,
    val: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Lazy<T> {
    /// Create a new lazy item
    pub const fn new(func: fn() -> T) -> Self {
        Self {
            init: AtomicBool::new(false),
            func,
            val: UnsafeCell::new(MaybeUninit::zeroed()),
        }
    }

    pub fn get(&self) -> &T {
        if self.init.load(Ordering::Acquire) {
            unsafe { self.get_ref() }
        } else {
            unsafe {
                self.eval();
                self.get_ref()
            }
        }
    }

    const unsafe fn get_ref(&self) -> &T {
        &*(self.val.get() as *mut T)
    }

    /// Evaluate the lazy item
    pub unsafe fn eval(&self) {
        (self.val.get() as *mut T).write((self.func)());
        self.init.store(true, Ordering::Release);
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

unsafe impl<T> Sync for Lazy<T> where T: Sync {}

macro_rules! lazy_static {
    ($(#[$outer_meta:meta])* $visib:vis lazy static $ident:ident: $ty:ty = $func:block;) => {
        fn create_static_item() -> $ty {
            $func
        }

        $(#[$outer_meta])*
        $visib static $ident: $crate::sync::Lazy<$ty> = $crate::sync::Lazy::new(create_static_item);
    };
}

pub(crate) use lazy_static;
