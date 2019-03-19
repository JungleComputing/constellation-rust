#[derive(Debug, Clone)]
pub struct Context {
    pub label: String,
}

impl Context {
    fn new(label: String) -> Context {
        Context { label }
    }
}
