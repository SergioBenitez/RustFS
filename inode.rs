#![feature(globs)]

extern crate rand;
extern crate time;

use time::Timespec;
use std::mem;
use std::slice::MutableCloneableVector;

static PAGE_SIZE: uint = 4096;
static LIST_SIZE: uint = 256;

type Page = Box<([u8, ..PAGE_SIZE])>;
type Entry = Option<Page>;
type TList<T> = Box<([T, ..LIST_SIZE])>;
type EntryList = TList<Entry>;
type DoubleEntryList = TList<Box<EntryList>>;

#[inline(always)]
fn ceil_div(x: uint, y: uint) -> uint {
  return (x + y - 1) / y;
}

pub struct Inode {
  single: EntryList, // Box<([Option<Page>, ..256])>
  // data2: DoubleEntryList,
  size: uint,

  mod_time: Timespec,
  access_time: Timespec,
  create_time: Timespec,
}

impl Inode {
  pub fn new() -> Inode {
    let time_now = time::get_time();

    // Because Rust is a pain in the ass to work with
    let mut entries: EntryList = box unsafe { mem::uninit() }; 
    for x in entries.mut_iter() { unsafe { mem::move_val_init(x, None); } };

    Inode {
      single: entries,
      size: 0,

      mod_time: time_now,
      access_time: time_now,
      create_time: time_now
    }
  }

  fn get_or_alloc_page<'a>(&'a mut self, num: uint) -> &'a mut Page {
    let page = &mut self.single[num];
    match page {
      &None => {
        *page = Some(box () ([0u8, ..4096]));
        page.get_mut_ref()
      },
      &Some(_) => page.get_mut_ref()
    }
  }

  fn get_page<'a>(&'a self, num: uint) -> &'a Option<Page> {
    &self.single[num]
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

      // // Need to account for offsets from first and last blocks
      let num_bytes = if i == blocks_to_act_on - 1 {
        data.len() - written
      } else {
        PAGE_SIZE - block_offset
      };

      // Finding our block, writing to it
      let page = self.get_or_alloc_page(start + i);
      let slice = page.mut_slice(block_offset, block_offset + num_bytes);
      written += slice.copy_from(data.slice(written, written + num_bytes));
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

      // // Need to account for offsets from first and last blocks
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
      read += slice.copy_from(page.slice(block_offset, block_offset + num_bytes));
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

  use super::*;
  use std::default::Default;
  use rand::random;
  
  fn rand_array(size: uint) -> Vec<u8> {
    Vec::from_fn(size, |_| {
      random::<u8>()
    })
  }

  #[test]
  fn test_simple_write() {
    static size: uint = 4096 * 8 + 3434;

    let original_data = rand_array(size);
    let mut inode = Inode::new();
    let mut buf = [0u8, ..size];

    // Write the random data, read it back into buffer
    inode.write(0, original_data.as_slice());
    inode.read(0, buf);

    // Make sure inode is right size
    assert_eq!(size, inode.size());

    // Make sure contents are correct
    for i in range(0, size) {
      println!("buf: {}, actual: {}", buf[i], original_data.get(i));
      assert_eq!(buf[i], *original_data.get(i));
    }
  }
}

fn main() { }
