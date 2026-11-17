pub mod mapping;
pub mod profiles;
pub trait OutputBackend {
    fn available(&self) -> bool;
    fn stop(&mut self);
}
pub struct UnsupportedOutput;
impl OutputBackend for UnsupportedOutput {
    fn available(&self) -> bool {
        false
    }
    fn stop(&mut self) {}
}
