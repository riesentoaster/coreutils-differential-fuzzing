pub trait Executor {
    fn execute(&mut self, i: usize, path: &str);
    fn setup(&self) {}
    fn teardown(&self) {}
}
