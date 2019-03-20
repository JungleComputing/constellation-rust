pub trait MessageTrait: Sync + Send {
    fn to_string(&self) -> &String;
}