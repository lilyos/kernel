/// The `Init` trait
/// Minimal initializations may be represented merely by
/// not implementing `Init::init(&self, init_val: Self::Input)`
///
/// # Example
/// ```rust
/// pub struct Table {
///     seats: Mutex<usize>,
/// }
///
/// pub enum TableError {
///     NotMahogany,
/// }
///
/// impl Init for Table {
///     type Error = TableError;
///
///     type Input = usize;
///
///     /// Initialize the Table
///     /// If the seat count isn't evenly divisible by three, this errors out,
///     /// as the guests will only accept THE FINEST MAHOGANY if their party size is a multiple of three
///     fn init(&self, init_val: Self::Input) -> Result<(), Self::Error> {
///         if init_val % 3 != 0 {
///             Err(TableError::NotMahogany)    
///         } else {
///             let mut data = self.seats.lock().unwrap();
///             *data = init_val;
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Init {
    /// The error to be returned by the init implementation, defaults to [core::convert::Infallible]
    type Error = core::convert::Infallible;

    /// The input type for the `init` function
    type Input = ();

    /// The function that actually does the initialization
    fn init(&self, val: Self::Input) -> Result<(), Self::Error> {
        let _ = val;
        Ok(())
    }
}
