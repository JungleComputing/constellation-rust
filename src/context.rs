use std::fmt;

/// Holds any number of context, use to identify on which executor
/// activities should be executed. Lives both in Constellation and on each
/// activity.
///
/// # Members
/// * `context_vec` - A vector of Contexts
#[derive(Debug, Clone)]
pub struct ContextVec {
    pub context_vec: Vec<Context>,
}

impl ContextVec {
    pub fn new() -> ContextVec {
        ContextVec {
            context_vec: Vec::new(),
        }
    }

    pub fn append(&mut self, context: &Context) {
        self.context_vec.push(context.clone());
    }

    pub fn remove(&mut self, context: &Context) {
        self.context_vec.iter().map(|x| x != context);
    }

    pub fn contains(&self, context: &Context) -> bool {
        self.context_vec.contains(context)
    }
}

impl fmt::Display for ContextVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp: &Vec<Context> = self.context_vec.as_ref(); //.into_iter().map(|x|x.label).collect();
        let labels: Vec<String> = tmp.into_iter().map(|x| x.label.clone()).collect();

        write!(f, "context:{:?}", labels)
    }
}

/// Context used to identify where an activity should be executed.
#[derive(Debug, Clone)]
pub struct Context {
    pub label: String,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "context:{}", self.label)
    }
}

impl PartialEq for Context {
    fn eq(&self, other: &Context) -> bool {
        self.label == other.label
    }
}

impl Eq for Context {}
