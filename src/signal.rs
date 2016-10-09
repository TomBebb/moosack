pub struct Signal<T> {
    listeners: Vec<Box<Fn(T) + 'static>>,
}
impl<T> Signal<T> {
    pub fn new() -> Signal<T> {
        Signal { listeners: Vec::new() }
    }
    pub fn bind<F>(&mut self, func: F)
        where F: Fn(T) + 'static
    {
        self.listeners.push(Box::new(func));
    }
    pub fn invoke(&self, value: T)
        where T: Clone
    {
        for l in &self.listeners {
            l(value.clone());
        }
    }
}
