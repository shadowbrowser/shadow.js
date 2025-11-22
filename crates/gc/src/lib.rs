pub mod trace;

use crate::trace::Trace;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ptr::NonNull;

// Thread-local heap to avoid passing context everywhere
thread_local! {
    static HEAP: RefCell<Heap> = RefCell::new(Heap::new());
}

trait Traceable {
    fn trace(&self, visited: &mut HashSet<usize>);
    fn set_marked(&self, marked: bool);
    fn is_marked(&self) -> bool;
    fn as_ptr(&self) -> usize;
}

struct GcBox<T: ?Sized> {
    marked: Cell<bool>,
    data: RefCell<T>,
}

impl<T: Trace + ?Sized> Traceable for GcBox<T> {
    fn trace(&self, visited: &mut HashSet<usize>) {
        if !self.marked.get() {
            self.marked.set(true);
            self.data.borrow().trace(visited);
        }
    }

    fn set_marked(&self, marked: bool) {
        self.marked.set(marked);
    }

    fn is_marked(&self) -> bool {
        self.marked.get()
    }

    fn as_ptr(&self) -> usize {
        self as *const Self as *const () as usize
    }
}

pub struct Heap {
    objects: Vec<Box<dyn Traceable>>,
    bytes_allocated: usize,
    threshold: usize,
}

impl Heap {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
            bytes_allocated: 0,
            threshold: 1024 * 1024, // 1MB start
        }
    }

    fn alloc<T: Trace + 'static>(&mut self, value: T) -> Gc<T> {
        let gc_box = Box::new(GcBox {
            marked: Cell::new(false),
            data: RefCell::new(value),
        });

        let ptr_raw = &*gc_box as *const GcBox<T>;
        let ptr = NonNull::new(ptr_raw as *mut GcBox<T>).unwrap();

        self.objects.push(gc_box);

        Gc { ptr }
    }

    fn collect(&mut self, roots: &[&dyn Trace]) {
        let mut visited = HashSet::new();

        // 1. Unmark all
        for obj in &self.objects {
            obj.set_marked(false);
        }

        // 2. Mark roots
        for root in roots {
            root.trace(&mut visited);
        }

        // 3. Sweep
        self.objects.retain(|obj| obj.is_marked());
        let after = self.objects.len();

        // Adjust threshold
        if after > self.threshold {
            self.threshold = after * 2;
        }
    }
}

#[derive(Debug)]
pub struct Gc<T: ?Sized> {
    ptr: NonNull<GcBox<T>>,
}

impl<T: Trace + 'static> Trace for Gc<T> {
    fn trace(&self, visited: &mut HashSet<usize>) {
        let ptr_val = self.ptr.as_ptr() as *const () as usize;
        if visited.insert(ptr_val) {
            unsafe {
                let gc_box = self.ptr.as_ref();
                gc_box.set_marked(true);
                gc_box.data.borrow().trace(visited);
            }
        }
    }
}

impl<T: Trace + 'static> Gc<T> {
    pub fn new(value: T) -> Self {
        HEAP.with(|heap| heap.borrow_mut().alloc(value))
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, T> {
        unsafe { self.ptr.as_ref().data.borrow() }
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, T> {
        unsafe { self.ptr.as_ref().data.borrow_mut() }
    }
}

impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T: ?Sized> Copy for Gc<T> {}

impl<T: ?Sized> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.ptr.as_ptr() as *const ()) == (other.ptr.as_ptr() as *const ())
    }
}

pub struct GC;

impl GC {
    pub fn new() -> Self {
        Self {}
    }
    pub fn collect(&mut self, roots: &[&dyn Trace]) {
        HEAP.with(|heap| {
            heap.borrow_mut().collect(roots);
        })
    }
}
