#![allow(raw_pointer_deriving)]
#![crate_type = "lib"]

/**
 * A growing (not yet shrinking), typed slab allocator.
 *
 * To use:
 *
 * let s = SlabAllocator::new(10);
 * {
 *   let mut first = s.alloc(0); // Type is SlabBox<'a, int>
 *   *first = 10;
 *   assert_eq!(*first, 10);
 *
 *   let mut second = s.alloc(0);
 *   *second = 20;
 *   assert_eq!(*first, 20);
 *   assert_eq!(*second, *first);
 *
 *   let third = first.clone(); // Referencing same as first.
 *   assert!(*first == *third);
 *
 *   *second = 30;
 *   assert_eq!(*first == 30);
 *   assert_eq!(*first == *third);
 *   assert_eq!(*second == *third);
 * } // first, second, third returned to allocator.
 */

extern crate libc;

use std::mem;
use std::mem::transmute;
use std::rc::Rc;
use libc::{size_t, malloc};
use std::cell::{Cell, RefCell};
use std::intrinsics;

#[deriving(Show)]
struct Slab<'a, T> {
  parent: &'a SlabAllocator<T>, // Would RC be better to get rid of 'a? How to?
  ptr: *mut T
}

impl<'a, T> Slab<'a, T> {
  fn borrow<'r>(&'r self) -> &'r T {
    unsafe { &*self.ptr }
  }

  fn borrow_mut<'r>(&'r mut self) -> &'r mut T {
    unsafe { &mut*self.ptr }
  }
}

impl<'a, T: Eq> Eq for Slab<'a, T> {
  fn eq(&self, other: &Slab<T>) -> bool {
    self.borrow() == other.borrow()
  }
}

#[unsafe_destructor]
impl<'a, T> Drop for Slab<'a, T> {
  fn drop(&mut self) {
    self.parent.free(self.ptr);
  }
}

#[deriving(Clone)]
pub struct SlabBox<'a, T>(Rc<RefCell<Slab<'a, T>>>);

impl<'a, T> SlabBox<'a, T> {
  #[inline(always)]
  pub fn borrow<'r>(&'r self) -> &'r T {
    let SlabBox(ref rc) = *self;
    unsafe { transmute(rc.borrow().borrow()) }
  }

  #[inline(always)]
  pub fn borrow_mut<'r>(&'r mut self) -> &'r mut T {
    let SlabBox(ref rc) = *self;
    unsafe { transmute(rc.borrow_mut().borrow_mut()) }
  }
}

impl<'a, T> Deref<T> for SlabBox<'a, T> {
  #[inline(always)]
  fn deref<'r>(&'r self) -> &'r T {
    self.borrow()
  }
}

impl<'a, T> DerefMut<T> for SlabBox<'a, T> {
  #[inline(always)]
  fn deref_mut<'r>(&'r mut self) -> &'r mut T {
    self.borrow_mut()
  }
}

#[deriving(Show)]
pub struct SlabAllocator<T> {
  items: Vec<*mut T>,       // holds pointers to allocations
  alloc: Cell<uint>,        // number of outstanding items
  capacity: Cell<uint>      // number of items pre-allocated (valid in items)

}

impl<T> SlabAllocator<T> {
  pub fn new(initial_size: uint) -> SlabAllocator<T> {
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

  /**
   * TODO: Have a way to return possibly dirty objects.
   *
   * Idea: value of type Option<T>. Then, if Some, use the value inside as the
   * initial value of the slab allocated object. If none, give unclean memory.
   * 
   * Alternatively, have an alloc_dirty that always returns possibly dirty
   * values.
   */
  pub fn alloc<'r>(&'r self, value: T) -> SlabBox<'r, T> {
    let (alloc, capacity) = (self.alloc.get(), self.capacity.get());
    if alloc >= capacity {
      unsafe {
        // is there a safe/better way to do something like this?
        let mut_self: &mut SlabAllocator<T> = transmute(self);
        mut_self.expand(capacity * 2 - capacity);
      }
    }

    let ptr: *mut T = *self.items.get(alloc);
    unsafe { mem::overwrite(&mut *ptr, value); }

    self.alloc.set(alloc + 1);
    let slab = Slab { parent: self, ptr: ptr }; 
    SlabBox(Rc::new(RefCell::new(slab)))
  }

  fn free(&self, ptr: *mut T) {
    let alloc = self.alloc.get();
    if alloc <= 0 { fail!("Over-freeing....somehow"); }

    self.alloc.set(alloc - 1);
    unsafe {
      // Dropping if needed
      if intrinsics::needs_drop::<T>() {
        let ty = intrinsics::get_tydesc::<T>();
        ((*ty).drop_glue)(ptr as *i8);
      }

      // Letting an immutable slice be mutable, unsafely
      let items: &mut [*mut T] = transmute(self.items.as_slice());
      items[alloc - 1] = ptr;
    }
  }

  pub fn stats(&self) -> (uint, uint) {
    (self.alloc.get(), self.capacity.get())
  }
}

#[cfg(test)]
mod tests {
  extern crate test;
  use super::{SlabAllocator, SlabBox};
  use std::mem;

  // Used to test that deallocation works.
  #[deriving(Eq, Show, Clone)]
  struct MyThing {
    field1: Option<Box<int>>,
    done: bool
  }

  impl Drop for MyThing {
    fn drop(&mut self) {
      println!("Being dropped.");
      if !self.done { fail!("Should not be dropped: {}", self.done); }
    }
  }

  #[test]
  fn test_one_mut_alloc() {
    let slab_allocator = SlabAllocator::new(20);
    let object = slab_allocator.alloc(239);
    assert_eq!(*object, 239);
    assert_eq!(*object, *slab_allocator.alloc(239));
  }

  #[test]
  fn test_one_struct_alloc() {
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
  #[should_fail]
  fn test_struct_dealloc() {
    let slab_allocator = SlabAllocator::new(20);
    {
      let struct_obj = MyThing { field1: Some(box 20), done: false };
      let object = slab_allocator.alloc(struct_obj);
      assert_eq!(object.field1, Some(box 20));
    }
    // Will cause double-fail if deallocation didn't occur.
    fail!("Did not dallocate struct in time.");
  }

  #[test]
  #[should_fail]
  fn test_box_struct_dealloc() {
    let slab_allocator = SlabAllocator::new(20);
    {
      let struct_obj = box MyThing { field1: Some(box 2445), done: false };
      let object = slab_allocator.alloc(struct_obj);
      assert_eq!(object.field1, Some(box 2445));
    }
    // Will cause double-fail if deallocation didn't occur.
    fail!("Did not dallocate struct in time.");
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
  fn test_mut_box_alloc() {
    let slab_allocator = SlabAllocator::new(20);
    let mut object = slab_allocator.alloc(box 1111);
    assert_eq!(*object, box 1111);
    assert_eq!(*object, *slab_allocator.alloc(box 1111));

    let object2 = object.clone();
    assert_eq!(*object, *object2);
    let init_ptr: *int;
    unsafe {
      let ptr1: *int = mem::transmute(object.deref());
      let ptr2: *int = mem::transmute(object2.deref());
      init_ptr = ptr1;
      assert_eq!(ptr1, ptr2); // make sure we're not allocating new internal box
    }

    *object = box 2222;
    assert_eq!(*object, *object2);
    assert_eq!(*object2, box 2222);
    assert_eq!(**object2, 2222);

    let mut object3 = object2.clone();
    *object3 = box 3333;
    assert_eq!(*object, box 3333);
    assert_eq!(*object2, *object);
    assert_eq!(*object, *object3);

    unsafe {
      let ptr1: *int = mem::transmute(object.deref());
      let ptr2: *int = mem::transmute(object2.deref());
      let ptr3: *int = mem::transmute(object3.deref());
      assert_eq!(ptr1, ptr2); 
      assert_eq!(ptr2, ptr3);

      // this one is a bit interesting. after doing some tests, it looks like if
      // you have a let mut a: Box<thing> and then you do a = box newthing, the
      // box will continue being the same one and only the content inside will
      // change, which is pretty cool.
      assert_eq!(init_ptr, ptr1); // because they're boxes, ptr shouldn't change
    }
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
      // Making sure nothing's in there
      let (alloc, _) = slab_allocator.stats();
      assert_eq!(alloc, 0);

      let object = slab_allocator.alloc(i);
      assert_eq!(*object, i);
    }

    // Just in case some weird business is happenning
    for i in range(-239, -180) {
      let object = slab_allocator.alloc(i);
      assert_eq!(*object, i);
    }

    // Should still be empty
    let (alloc, _) = slab_allocator.stats();
    assert_eq!(alloc, 0);
  }

  #[test]
  fn test_over_alloc() {
    // This test should pass when allocator can grow.
    let slab_allocator = SlabAllocator::new(20);

    // Alloacting more then the capacity
    // Testing reference counting (by using clone), shouldn't drop
    let mut vec = Vec::new();
    for i in range(0, 100) {
      let obj = slab_allocator.alloc(i);
      assert_eq!(*obj, i);
      vec.push(obj.clone());
    }

    // Make sure the allocator performed as expected
    let (allocated, capacity) = slab_allocator.stats();
    assert_eq!(allocated, 100);
    assert_eq!(capacity, 160); // 20 -> 40 -> 80 -> 160

    // Get rid of all the references to the objects
    vec.truncate(0);

    // Make sure they were deallocated
    let (allocated, capacity) = slab_allocator.stats();
    assert_eq!(allocated, 0);
    assert_eq!(capacity, 160); // 20 -> 40 -> 80 -> 160
  }

  #[test]
  fn test_copy_return_alloc() {
    let slab_allocator = SlabAllocator::new(20);

    // Drop should be called for each object after each loop since we're only
    // storing the value of the object and not the object itself
    let mut vec = Vec::new();
    for i in range(0, 25) {
      let (alloc, _) = slab_allocator.stats();
      assert_eq!(alloc, 0);

      let obj = slab_allocator.alloc(i);
      assert_eq!(*obj, i);
      vec.push(*obj);
    }

    let (alloc, _) = slab_allocator.stats();
    assert_eq!(alloc, 0);
  }

  #[test]
  fn test_usage_external_allocator() {
    struct MyThing<'r> {
      item: SlabBox<'r, int>,
      allocator: &'r SlabAllocator<int>,
    }

    impl<'r> MyThing<'r> {
      fn new(allocator: &'r SlabAllocator<int>, num: int) -> MyThing<'r> {
        MyThing {
          item: allocator.alloc(num),
          allocator: allocator,
        }
      }

      fn set_num(&mut self, num: int) {
        self.item = self.allocator.alloc(num);
      }
    }

    let allocator = SlabAllocator::new(10);
    let mut thing = MyThing::new(&allocator, 120);
    let thing2 = MyThing::new(&allocator, 130);

    assert_eq!(*thing.item, 120);
    assert_eq!(*thing2.item, 130);

    let (alloc, _) = allocator.stats();
    assert_eq!(alloc, 2);

    thing.set_num(434);
    assert_eq!(*thing.item, 434);
    assert_eq!(*thing2.item, 130);

    let (alloc, _) = allocator.stats();
    assert_eq!(alloc, 2);
  }

  // Not sure if this is possible.
  #[test]
  fn test_usage_internal_allocator() {
    use std::cell::RefCell;

    struct MyThing<'r> {
      item: RefCell<Option<SlabBox<'r, int>>>,
      allocator: SlabAllocator<int>
    }

    impl<'r> MyThing<'r> {
      fn new() -> MyThing {
        let thing = MyThing {
          item: RefCell::new(None),
          allocator: SlabAllocator::new(10)
        };

        thing
      }

      fn set(&'r self, num: int) {
        *self.item.borrow_mut() = Some(self.allocator.alloc(num));
      }

      fn item(&'r self) -> SlabBox<'r, int> {
        self.item.borrow().get_ref().clone()
      }
    }

    // Ideally, this would work.
    // let thing = MyThing::new(120);
    // assert!(**thing.item() == 120);
    
    let thing = MyThing::new();

    thing.set(127);
    assert!(*thing.item() == 127);

    thing.set(50);
    assert!(*thing.item() == 50);

    let (alloc, _) = thing.allocator.stats();
    assert_eq!(alloc, 1);

    let oldthing = thing.item();
    thing.set(120);
    assert_eq!(*oldthing, 50);
    assert_eq!(*thing.item(), 120);

    let (alloc, _) = thing.allocator.stats();
    assert_eq!(alloc, 2);
  }
}
