///! Use the steal strategy to specify if a constellation instance should
///! prioritize large jobs or small jobs primarily when stealing and
///! distributing them.
///!
///! TODO This is not yet fully implemented in the thread_helper

#[derive(Clone)]
pub enum StealStrategy {
    SMALLEST,
    BIGGEST
}
