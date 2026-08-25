pub trait SimpleContext: Send + Sync {
    fn ready(&self) -> bool {
        true
    }
}
