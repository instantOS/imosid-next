pub trait Hashable {
    fn finalize(&mut self);
    fn compile(&mut self) -> bool;
}

