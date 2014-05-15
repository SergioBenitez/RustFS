#![allow(raw_pointer_deriving)]

/**
 * A growing (not yet shrinking), typed slab allocator.
 *
 * To use:
 *
 * let s = SlabAllocator::new();
 * {
 *   let first = s.alloc(0); // Type is Rc<Slab<int>>;
 *   *first = 10;
 *
 *   let second = s.alloc(0);
 *   *second = 20;
 *
 *   let third = first.clone(); // Referencing same as first.
 *   assert!(*first == *third);
 *
 *   *third = 30;
 *   assert!(*third == 30);
 *   assert!(*first == *third);
 * } // Both first and second returned to allocator.
 *
 * How to do destruction of allocator and its slabs?
 *
 * Idea: Keep an unsafe pointer to the parent SlabAllocator (or perhaps a weak
 * RC link?). On drop of slab, check if the allocator is valid, somehow. If it 
 * is not, do nothing, if it is, return the slab. If the SlabAllocator is
 * dropped, then deallocate all of its memory.
 *
 * How to check if allocator is still alive? If the allocator keeps track of
 * all of its slabs, it can set a bit in the slab when it is deallocated.
 */

extern crate libc;

use std::mem;
use std::mem::transmute;
use std::rc::Rc;
use libc::{size_t, malloc};
use std::cell::{Cell, RefCell};

#[deriving(Show)]
struct Slab<'a, T> {
  parent: &'a SlabAllocator<T>,
  ptr: *mut T
}

impl<'a, T: Send> Slab<'a, T> {
  pub fn borrow<'r>(&'r self) -> &'r T {
    unsafe { &*self.ptr }
  }

  pub fn borrow_mut<'r>(&'r mut self) -> &'r mut T {
    unsafe { &mut*self.ptr }
  }
}

impl<'a, T: Eq + Send> Eq for Slab<'a, T> {
  fn eq(&self, other: &Slab<T>) -> bool {
    self.borrow() == other.borrow()
  }
}

#[unsafe_destructor]
impl<'a, T: Send> Drop for Slab<'a, T> {
  fn drop(&mut self) {
    println!("I should be returned back to {:?}.", self.parent);
    self.parent.free(self.ptr);
  }
}

#[deriving(Clone)]
struct SlabBox<'a, T>(Rc<RefCell<Slab<'a, T>>>);

impl<'a, T: Send> SlabBox<'a, T> {
  #[inline(always)]
  fn borrow<'r>(&'r self) -> &'r T {
    let SlabBox(ref rc) = *self;
    unsafe { transmute(rc.borrow().borrow()) }
  }

  #[inline(always)]
  fn borrow_mut<'r>(&'r self) -> &'r mut T {
    let SlabBox(ref rc) = *self;
    unsafe { transmute(rc.borrow_mut().borrow_mut()) }
  }
}

impl<'a, T: Send> Deref<T> for SlabBox<'a, T> {
  #[inline(always)]
  fn deref<'r>(&'r self) -> &'r T {
    self.borrow()
  }
}

impl<'a, T: Send> DerefMut<T> for SlabBox<'a, T> {
  #[inline(always)]
  fn deref_mut<'r>(&'r mut self) -> &'r mut T {
    self.borrow_mut()
  }
}

#[deriving(Show)]
struct SlabAllocator<T> {
  items: Vec<*mut T>,       // holds pointers to allocations
  alloc: Cell<uint>,        // number of outstanding items
  capacity: Cell<uint>      // number of items pre-allocated (valid in items)

}

impl<T: Send> SlabAllocator<T> {
  fn new(initial_size: uint) -> SlabAllocator<T> {
    let mut allocator = SlabAllocator {
      items: Vec::with_capacity(initial_size),
      alloc: Cell::new(0),
      capacity: Cell::new(0)
    };

    allocator.expand(initial_size);
    allocator
  }
  
  // pre-allocates and additional new_items and adds them to the end of
  // self.items, increasing self.capacity with the new size
  fn expand(&mut self, new_items: uint) {
    unsafe {
      let memory = malloc((mem::size_of::<T>() * new_items) as size_t) as *mut T;
      assert!(!memory.is_null());

      for i in range(0, new_items as int) {
        self.items.push(memory.offset(i));
      }
    }

    self.capacity.set(self.capacity.get() + new_items);
  }

  fn alloc<'r>(&'r self, value: T) -> SlabBox<'r, T> {
    let alloc = self.alloc.get();
    if alloc >= self.capacity.get() { fail!("Out of memory."); }

    let ptr: *mut T = *self.items.get(alloc);
    unsafe { mem::move_val_init(&mut *ptr, value); }

    self.alloc.set(alloc + 1);
    let slab = Slab { parent: self, ptr: ptr }; 
    SlabBox(Rc::new(RefCell::new(slab)))
  }

  fn free(&self, ptr: *mut T) {
    let alloc = self.alloc.get();
    if alloc <= 0 { fail!("Over-freeing....somehow"); }

    self.alloc.set(alloc - 1);
    unsafe {
      // Letting an immutable slice be mutable, unsafely
      let items: &mut [*mut T] = transmute(self.items.as_slice());
      items[alloc - 1] = ptr;
    }
  }
}

#[cfg(test)]
mod tests {
  extern crate test;
  use super::SlabAllocator;

  #[test]
  fn test_one_mut_alloc() {
    let slab_allocator = SlabAllocator::new(20);
    let object = slab_allocator.alloc(239);
    assert_eq!(*object, 239);
    assert_eq!(*object, *slab_allocator.alloc(239));
  }

  #[test]
  fn test_one_struct_alloc() {
    #[deriving(Eq, Show, Clone)]
    struct MyThing {
      field1: Option<Box<int>>,
      done: bool
    }

    impl Drop for MyThing {
      fn drop(&mut self) {
        if !self.done { fail!("Should not be dropped: {}", self.done); }
      }
    }
    
    let slab_allocator = SlabAllocator::new(20);
    let struct_obj = MyThing { field1: Some(box 20), done: false };
    let mut object = slab_allocator.alloc(struct_obj);

    assert_eq!(object.field1, Some(box 20));
    object.done = true;

    // Now with a boxed type
    let slab_allocator = SlabAllocator::new(20);
    let struct_obj = box MyThing { field1: Some(box 40), done: false };
    let mut object = slab_allocator.alloc(struct_obj);

    assert_eq!(object.field1, Some(box 40)); // same as (*object).field1
    object.done = true;
  }

  #[test]
  fn test_two_allocs_with_boxes() {
    let slab_allocator = SlabAllocator::new(20);
    let object = slab_allocator.alloc(box 239);
    let object2 = slab_allocator.alloc(box 23089);
    let object3 = object.clone();

    assert!(**object == 239);
    assert_eq!(*object2, box 23089);
    assert!(*object2 != *object);
    assert!(*object2 != *object);
    assert_eq!(*object3, *object);
  }

  #[test]
  fn test_mut_alloc() {
    let slab_allocator = SlabAllocator::new(20);
    let mut object = slab_allocator.alloc(239);
    assert_eq!(*object, 239);

    *object = 500;
    assert_eq!(*object, 500);

    *object = 50;
    assert_eq!(*object, 50);
  }

  #[test]
  fn test_mut_alloc_clone() {
    let slab_allocator = SlabAllocator::new(20);
    let mut object = slab_allocator.alloc(239);
    let object2 = object.clone();
    let object3 = slab_allocator.alloc(77);

    assert_eq!(*object, 239);
    assert_eq!(*object, *object2);
    assert_eq!(*object3, 77);

    *object = 349;
    assert_eq!(*object, 349);
    assert_eq!(*object, *object2);
    assert_eq!(*object3, 77);
  }

  #[test]
  fn test_one_alloc_boxed() {
    let slab_allocator = box SlabAllocator::new(20);
    let object = slab_allocator.alloc(239);
    assert_eq!(*object, 239);
  }

  #[test]
  fn test_many_alloc() {
    let slab_allocator = SlabAllocator::new(20);

    // Alloacting and verifying 20 items and putting into vector.
    let mut vec = Vec::new();
    for i in range(0, 20) {
      let obj = slab_allocator.alloc(i);
      assert_eq!(*obj, i);
      vec.push(obj);
    }

    // Making sure they're all still there and different.
    let mut i = 0;
    for obj in vec.iter() {
      assert_eq!(**obj, i);
      i += 1;
    }
  }

  #[test]
  fn test_alloc_return() {
    let slab_allocator = box SlabAllocator::new(20);

    // Drop should be called for each object after each loop
    for i in range(0, 50) {
      let object = slab_allocator.alloc(i);
      assert_eq!(*object, i);
    }

    // Just in case some weird business is happenning
    for i in range(-239, -180) {
      let object = slab_allocator.alloc(i);
      assert_eq!(*object, i);
    }
  }

  #[test]
  #[should_fail]
  fn test_over_alloc() {
    // This test should pass when allocator can grow.
    let slab_allocator = SlabAllocator::new(20);

    // Alloacting more then the capacity
    // Testing reference counting (by using clone), shouldn't drop
    let mut vec = Vec::new();
    for i in range(0, 25) {
      let obj = slab_allocator.alloc(i);
      assert_eq!(*obj, i);
      vec.push(obj.clone());
    }
  }

  #[test]
  fn test_copy_return_alloc() {
    let slab_allocator = SlabAllocator::new(20);

    // Drop should be called for each object after each loop since we're only
    // storing the value of the object and not the object itself
    let mut vec = Vec::new();
    for i in range(0, 25) {
      let obj = slab_allocator.alloc(i);
      assert_eq!(*obj, i);
      vec.push(*obj);
    }
  }
}
