pub struct CachePool<T> {
    items: Vec<T>,
    in_use: usize,
}

impl<T> Default for CachePool<T> {
    fn default() -> Self {
        Self {
            items: vec![],
            in_use: 0,
        }
    }
}

impl<T> CachePool<T> {
    pub fn get<'a>(&'a mut self, ctor: impl FnOnce() -> T) -> &'a T {
        if self.in_use >= self.items.len() {
            self.items.push(ctor());
        }
        let i = self.in_use;
        self.in_use += 1;
        &self.items[i]
    }

    pub fn reset(&mut self) {
        self.in_use = 0;
    }
}
