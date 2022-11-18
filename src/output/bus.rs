pub trait Bus<T: Sized> {
    fn write(&mut self, data: T);
}
