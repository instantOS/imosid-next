pub enum CompileResult {
    Changed,
    Unchanged,
}

pub trait Hashable {
    fn finalize(&mut self);
    fn compile(&mut self) -> CompileResult;
}

impl From<CompileResult> for bool {
    fn from(result: CompileResult) -> Self {
        match result {
            CompileResult::Changed => true,
            CompileResult::Unchanged => false,
        }
    }
}
