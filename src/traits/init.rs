/// Trait for singletons that need to be initialized
pub trait Init {
    /// To be run before init
    fn pre_init(&mut self);

    /// The main initialization
    fn init(&mut self);

    /// To be run after initialization
    fn post_init(&mut self);
}
