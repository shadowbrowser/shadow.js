use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Gc<T: ?Sized>(Rc<RefCell<T>>);

impl<T> Gc<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }
}

impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

pub struct GC;

impl GC {
    pub fn new() -> Self { Self {} }
    pub fn collect(&mut self) {
        // In a real implementation, this would trace and sweep.
        // With Rc, we rely on reference counting.
        // Cycle detection would go here.
    }
}
