pub trait Init {
    fn pre_init(&mut self);

    fn init(&mut self);

    fn post_init(&mut self);
}
