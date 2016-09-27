use time;
use time::Timespec;
use std::mem;
use std::ptr;
use std::ptr::copy_nonoverlapping;

const PAGE_SIZE: usize = 4096;
const LIST_SIZE: usize = 256;

type Page = Box<([u8; PAGE_SIZE])>;
type Entry = Page;
type EntryList = TList<Entry>; // TODO: Option<TList> for lazy loading
type DoubleEntryList = TList<EntryList>;
pub type TList<T> = Box<([Option<T>; LIST_SIZE])>;

#[inline(always)]
fn ceil_div(x: usize, y: usize) -> usize {
  return (x + y - 1) / y;
}

#[inline(always)]
pub fn create_tlist<T>() -> TList<T> {
  let mut list: TList<T> = Box::new(unsafe { mem::uninitialized() });
  for x in list.iter_mut() { unsafe { ptr::write(x, None); } };
  list
}

pub struct Inode {
  single: EntryList, // Box<([Option<Page>, ..256])>
  double: DoubleEntryList, // Box<[Option<Box<([Option<Page>>, ..256])>, ..256]
  size: usize,

  mod_time: Timespec,
  access_time: Timespec,
  create_time: Timespec,
}

impl Inode {
  pub fn new() -> Inode {
    let time_now = time::get_time();

    Inode {
      single: create_tlist(),
      double: create_tlist(),
      size: 0,

      mod_time: time_now,
      access_time: time_now,
      create_time: time_now
    }
  }

  fn get_or_alloc_page<'a>(&'a mut self, num: usize) -> &'a mut Page {
    if num >= LIST_SIZE + LIST_SIZE * LIST_SIZE {
      panic!("Maximum file size exceeded!")
    };

    // Getting a pointer to the page
    let page = if num < LIST_SIZE {
      // if the page num is in the singly-indirect list
      &mut self.single[num]
    } else {
      // if the page num is in the doubly-indirect list. We allocate a new
      // entry list where necessary (*entry_list = ...)
      let double_entry = num - LIST_SIZE;
      let slot = double_entry / LIST_SIZE;
      let entry_list = &mut self.double[slot];

      match *entry_list {
        None => *entry_list = Some(create_tlist()),
        _ => { /* Do nothing */ }
      }

      let entry_offset = double_entry % LIST_SIZE;
      &mut entry_list.as_mut().unwrap()[entry_offset]
    };

    match *page {
      None => *page = Some(Box::new([0u8; 4096])),
      _ => { /* Do Nothing */ }
    }

    page.as_mut().unwrap()
  }

  fn get_page<'a>(&'a self, num: usize) -> &'a Option<Page> {
    if num >= LIST_SIZE + LIST_SIZE * LIST_SIZE {
      panic!("Page does not exist.")
    };

    if num < LIST_SIZE {
      &self.single[num]
    } else {
      let double_entry = num - LIST_SIZE;
      let slot = double_entry / LIST_SIZE;
      let entry_offset = double_entry % LIST_SIZE;
      let entry_list = &self.double[slot];

      match *entry_list {
        None => panic!("Page does not exist."),
        _ => &entry_list.as_ref().unwrap()[entry_offset]
      }
    }
  }

  pub fn write(&mut self, offset: usize, data: &[u8]) -> usize {
    let mut written = 0;
    let mut block_offset = offset % PAGE_SIZE; // offset from first block

    let start = offset / PAGE_SIZE; // first block to act on
    let blocks_to_act_on = ceil_div(block_offset + data.len(), PAGE_SIZE);

    for i in 0..blocks_to_act_on {
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
      let mut page = self.get_or_alloc_page(start + i);
      let slice = &mut page[block_offset..(block_offset + num_bytes)];
      // written += slice.copy_from(data.slice(written, written + num_bytes));
      unsafe {
        // TODO: This may be extremely slow! Use copy_nonoverlapping, perhaps.
        let src = data[written..(written + num_bytes)].as_ptr();
        copy_nonoverlapping(src, slice.as_mut_ptr(), num_bytes);
      }

      written += num_bytes;
    }

    let last_byte = offset + written;
    if self.size < last_byte { self.size = last_byte; }

    let time_now = time::get_time();
    self.mod_time = time_now;
    self.access_time = time_now;

    written
  }

  pub fn read(&self, offset: usize, data: &mut [u8]) -> usize {
    let mut read = 0;
    let mut block_offset = offset % PAGE_SIZE; // offset from first block
    let start = offset / PAGE_SIZE; // first block to act on
    let blocks_to_act_on = ceil_div(block_offset + data.len(), PAGE_SIZE);

    for i in 0..blocks_to_act_on {
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
        &None => panic!("Empty data."),
        &Some(ref pg) => pg
      };

      let slice = &mut data[read..(read + num_bytes)];
      // read += slice.copy_from(page.slice(block_offset,
      // block_offset + num_bytes));
      unsafe {
        // copy_from is extremely slow! use copy_memory instead
        let src = page[block_offset..(block_offset + num_bytes)].as_ptr();
        copy_nonoverlapping(src, slice.as_mut_ptr(), num_bytes);
      }

      read += num_bytes;
    }

    read
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn stat(&self) -> (Timespec, Timespec, Timespec) {
    (self.create_time, self.access_time, self.mod_time)
  }
}

#[cfg(test)]
mod tests {
  extern crate rand;

  use super::{Inode};
  use self::rand::random;
  use time;

  fn rand_array(size: usize) -> Vec<u8> {
    (0..size).map(|_| random::<u8>()).collect()
  }

  #[test]
  fn test_simple_write() {
    const SIZE: usize = 4096 * 8 + 3434;

    let original_data = rand_array(SIZE);
    let time_now = time::get_time();
    let mut inode = Inode::new();
    let mut buf = [0u8; SIZE];

    // Write the random data, read it back into buffer
    inode.write(0, original_data.as_slice());
    inode.read(0, &mut buf);

    // Make sure inode is right size
    assert_eq!(SIZE, inode.size());

    // Make sure contents are correct
    for i in 0..SIZE {
      assert_eq!(buf[i], original_data[i]);
    }

    let (create, _, _) = inode.stat();
    assert_eq!(create.sec, time_now.sec);
  }
}
