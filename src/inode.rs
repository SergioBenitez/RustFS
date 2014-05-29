use time;
use time::Timespec;
use std::mem;
use slab::{SlabBox};
use super::GlobalAllocators;

static PAGE_SIZE: uint = 4096;
static LIST_SIZE: uint = 256;

pub type RawPage = [u8, ..PAGE_SIZE];
type Page<'a> = SlabBox<'a, RawPage>;
type Entry<'a> = Page<'a>;
type TList<T> = Box<([Option<T>, ..LIST_SIZE])>;
type EntryList<'a> = TList<Entry<'a>>; // TODO: Option<TList> for lazy loading
type DoubleEntryList<'a> = TList<EntryList<'a>>;

#[inline(always)]
fn ceil_div(x: uint, y: uint) -> uint {
  return (x + y - 1) / y;
}

macro_rules! expand(
  ($item:expr | $num:expr) => ({
    $item, expand!($item | $num - 1)
  });

  ($item:expr, $num:expr) => ({
    [$item, expand!($item | $num - 1)]
  });
)

#[inline(always)]
pub fn create_tlist<T>() -> TList<T> {
  let x: TList<T> = box () ([None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>, None::<T>]);
  x

//   let mut list: TList<T> = box unsafe { mem::uninitialized() }; 
//   for x in list.mut_iter() { unsafe { mem::overwrite(x, None); } };

//   list
}

pub struct Inode<'r> {
  single: EntryList<'r>, // Box<[Option<SlabBox<RawPage>>, ..256]>
  double: DoubleEntryList<'r>, // Box<[Option<EntryList>, ..256]>
  size: uint,

  mod_time: Timespec,
  access_time: Timespec,
  create_time: Timespec,

  allocators: &'r GlobalAllocators<'r>
}

impl<'r> Inode<'r> {
  pub fn new(allocators: &'r GlobalAllocators<'r>) -> Inode<'r> {
    use std::mem::{size_of_val, size_of};

    let time_now = time::get_time();

    Inode {
      single: create_tlist(),
      double: create_tlist(),
      size: 0,

      mod_time: time_now,
      access_time: time_now,
      create_time: time_now,

      allocators: allocators
    }
  }

  fn get_or_alloc_page<'a>(&'a mut self, num: uint) -> &'a mut Page<'r> {
    if num >= LIST_SIZE + LIST_SIZE * LIST_SIZE {
      fail!("Maximum file size exceeded!")
    };
  
    // Getting a pointer to the page
    let page = if num < LIST_SIZE {
      // if the page num is in the singly-indirect list
      &mut self.single[num]
    } else {
      // if the page num is in the doubly-indirect list. We allocate a new
      // entry list where necessary (*entry_list = ...)
      let doubleEntry = num - LIST_SIZE;
      let slot = doubleEntry / LIST_SIZE;
      let entry_list = &mut self.double[slot];

      match entry_list {
        &None => *entry_list = Some(create_tlist()),
        _ => { /* Do nothing */ }
      }
      
      let entry_offset = doubleEntry % LIST_SIZE;
      &mut entry_list.get_mut_ref()[entry_offset]
    };

    match page {
      // &None => *page = Some(box () ([0u8, ..4096])),
      &None => *page = Some(self.allocators.page.dirty_alloc()),
      _ => { /* Do Nothing */ }
    }

    page.get_mut_ref()
  }

  fn get_page<'a>(&'a self, num: uint) -> &'a Option<Page<'r>> {
    if num >= LIST_SIZE + LIST_SIZE * LIST_SIZE {
      fail!("Page does not exist.")
    };

    if num < LIST_SIZE {
      &self.single[num]
    } else {
      let doubleEntry = num - LIST_SIZE;
      let slot = doubleEntry / LIST_SIZE;
      let entry_offset = doubleEntry % LIST_SIZE;
      let entry_list = &self.double[slot];

      match entry_list {
        &None => fail!("Page does not exist."),
        _ => &entry_list.get_ref()[entry_offset]
      }
    }
  }

  pub fn write(&mut self, offset: uint, data: &[u8]) -> uint {
    let mut written = 0;
    let mut block_offset = offset % PAGE_SIZE; // offset from first block
    let start = offset / PAGE_SIZE; // first block to act on
    let blocks_to_act_on = ceil_div(block_offset + data.len(), PAGE_SIZE);

    for i in range(0, blocks_to_act_on) {
      // Resetting the block offset after first pass since we want to read from
      // the beginning of the block after the first time.
      if block_offset != 0 && i > 0 { block_offset = 0 };

      // Need to account for offsets from first and last blocks
      let num_bytes = if i == blocks_to_act_on - 1 {
        data.len() - written
      } else {
        PAGE_SIZE - block_offset
      };

      // Finding our block, writing to it
      let page = self.get_or_alloc_page(start + i);
      let slice = page.mut_slice(block_offset, block_offset + num_bytes);
      // written += slice.copy_from(data.slice(written, written + num_bytes));
      unsafe { 
        // copy_from is extremely slow! use copy_memory instead
        slice.copy_memory(data.slice(written, written + num_bytes));
        written += num_bytes;
      }
    }

    let last_byte = offset + written;
    if self.size < last_byte { self.size = last_byte; }

    written
  }

  pub fn read(&self, offset: uint, data: &mut [u8]) -> uint {
    let mut read = 0;
    let mut block_offset = offset % PAGE_SIZE; // offset from first block
    let start = offset / PAGE_SIZE; // first block to act on
    let blocks_to_act_on = ceil_div(block_offset + data.len(), PAGE_SIZE);

    for i in range(0, blocks_to_act_on) {
      // Resetting the block offset after first pass since we want to read from
      // the beginning of the block after the first time.
      if block_offset != 0 && i > 0 { block_offset = 0 };

      // Need to account for offsets from first and last blocks
      let num_bytes = if i == blocks_to_act_on - 1 {
        data.len() - read
      } else {
        PAGE_SIZE - block_offset
      };

      // Finding our block, reading from it
      let page = match self.get_page(start + i) {
        &None => fail!("Empty data."),
        &Some(ref pg) => pg
      };

      let slice = data.mut_slice(read, read + num_bytes);
      // read += slice.copy_from(page.slice(block_offset,
      // block_offset + num_bytes));
      unsafe { 
        // copy_from is extremely slow! use copy_memory instead
        slice.copy_memory(page.slice(block_offset, block_offset + num_bytes));
        read += num_bytes;
      }
    }

    read
  }

  pub fn size(&self) -> uint {
    self.size
  }
}

#[cfg(test)]
mod tests {
  extern crate rand;
  extern crate slab;

  use super::{Inode};
  use slab::SlabAllocator;
  use rand::random;
  use super::super::create_allocators;
  
  fn rand_array(size: uint) -> Vec<u8> {
    Vec::from_fn(size, |_| {
      random::<u8>()
    })
  }

  #[test]
  fn test_simple_write() {
    static size: uint = 4096 * 8 + 3434;

    let original_data = rand_array(size);
    let allocators = create_allocators();
    let mut inode = Inode::new(&allocators);
    let mut buf = [0u8, ..size];

    // Write the random data, read it back into buffer
    inode.write(0, original_data.as_slice());
    inode.read(0, buf);

    // Make sure inode is right size
    assert_eq!(size, inode.size());

    // Make sure contents are correct
    for i in range(0, size) {
      assert_eq!(buf[i], *original_data.get(i));
    }
  }
}
