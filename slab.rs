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
 */

extern crate libc;

use std::mem;
use std::mem::transmute;
use std::rc::Rc;
use libc::{size_t, malloc};
use std::cell::RefCell;

#[deriving(Show)]
struct Slab<T> {
  item: *mut T
}

impl<T> Deref<T> for Slab<T> {
  fn deref<'a>(&'a self) -> &'a T {
    unsafe { & *self.item }
  }
}

impl<T> DerefMut<T> for Slab<T> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    unsafe { &mut *self.item }
  }
}

impl<T> Eq for Slab<T> {
  fn eq(&self, other: &Slab<T>) -> bool {
    self.item == other.item
  }
}

#[unsafe_destructor]
impl<T> Drop for Slab<T> {
  fn drop(&mut self) {
    println!("I should be returned back to allocator.");
  }
}

#[deriving(Clone)]
struct SlabBox<T>(Rc<RefCell<Slab<T>>>);

impl<T> Deref<T> for SlabBox<T> {
  fn deref<'a>(&'a self) -> &'a T {
    unsafe {
      let cell = match self {
        &SlabBox(ref rc) => (rc.borrow())
      };
      transmute(cell.deref().deref())
    }
  }
}

impl<T> DerefMut<T> for SlabBox<T> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    unsafe {
      let mut cell = match self {
        &SlabBox(ref rc) => (rc.borrow_mut())
      };
      transmute(cell.deref_mut().deref_mut())
    }
  }
}

struct SlabAllocator<T> {
  items: Vec<*mut T>, // holds pointers to allocations
  alloc: uint,        // number of outstanding items
  capacity: uint      // number of items pre-allocated (valid in items)
}

impl<T> SlabAllocator<T> {
  fn new(initial_size: uint) -> SlabAllocator<T> {
    let mut allocator = SlabAllocator {
      items: Vec::with_capacity(initial_size),
      alloc: 0,
      capacity: initial_size
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
  }

  fn alloc(&mut self, value: T) -> SlabBox<T> {
    if self.alloc >= self.capacity { fail!("Out of memory."); }

    let item: *mut T = *self.items.get(self.alloc);
    unsafe {
      mem::move_val_init(&mut *item, value);
    }

    self.alloc += 1;
    let slab = Slab { item: item };
    SlabBox(Rc::new(RefCell::new(slab)))
  }
}

#[cfg(test)]
mod tests {
  extern crate test;
  use super::SlabAllocator;

  #[test]
  fn test_one_alloc() {
    let mut slab_allocator = SlabAllocator::new(20);
    let object = slab_allocator.alloc(239);
    assert_eq!(*object, 239);
  }

  #[test]
  fn test_two_allocs() {
    let mut slab_allocator = SlabAllocator::new(20);
    let object = slab_allocator.alloc(239);
    let object2 = slab_allocator.alloc(23089);
    let object3 = object.clone();

    assert!(*object == 239);
    assert_eq!(*object2, 23089);
    assert!(*object2 != *object);
    assert!(*object2 != *object);
    assert_eq!(*object3, *object);
  }

  #[test]
  fn test_mut_alloc() {
    let mut slab_allocator = SlabAllocator::new(20);
    let mut object = slab_allocator.alloc(239);
    assert_eq!(*object, 239);

    *object = 500;
    assert_eq!(*object, 500);

    *object = 50;
    assert!(*object != 500);
    assert_eq!(*object, 50);
  }

  #[test]
  fn test_mut_alloc_clone() {
    let mut slab_allocator = SlabAllocator::new(20);
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
}

// fn main() {
//   {
//     println!("Allocator...");
//     let mut slab_allocator = SlabAllocator::new(20);
//     println!("Object...");
//     let object = slab_allocator.alloc(54);
//     println!("Equal...");
//     assert_eq!(*object, 54);
//   }
//   println!("Done...");
// }
