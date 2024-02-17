pub trait DataBlob {
    fn get_initial_size() -> usize;
    fn get_size(&self) -> usize;
}
