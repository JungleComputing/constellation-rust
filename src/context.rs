use std::fmt;

#[derive(Debug, Clone)]
pub struct Context {
    pub label: String,
}

impl Context {
    fn new(label: String) -> Context {
        Context { label }
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "context:{}", self.label)
    }
}
