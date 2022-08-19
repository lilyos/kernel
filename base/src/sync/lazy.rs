use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
};

/// A type for lazy initialization
///
/// # Example
/// ```rust
/// fn initialize_the_number() -> u32 {
///     834234 << 3
/// }
///
/// let lazy_u32 = Lazy::new(initialize_the_number);
///
/// assert_eq!(*lazy_u32, 834234 << 3);
/// ```
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
        &*self.val.get().cast::<T>()
    }

    /// Evaluate the lazy item
    pub unsafe fn eval(&self) {
        self.val.get().cast::<T>().write((self.func)());
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

/// Create a lazy static
///
/// # Example
/// ```rust
/// lazy_static! {
///     pub lazy static OddTo100: Vec<i32> = {
///         let mut vec = Vec::new();
///         vec.extend(0..=100.filter(|i| i % 2 != 0));
///         vec
///     };
/// }
/// ```
macro_rules! lazy_static {
    (
        $(
            $(#[$outer_meta:meta])*
            $visib:vis lazy static $ident:ident: $ty:ty = $func:block;
        )*
    ) => {
        $(
            $(#[$outer_meta])*
            $visib static $ident: $crate::sync::Lazy<$ty> = {
                fn create_static_item() -> $ty {
                    $func
                }

                $crate::sync::Lazy::new(create_static_item)
            };
        )*
    };
}

pub(crate) use lazy_static;
