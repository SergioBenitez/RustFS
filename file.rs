extern crate time;
extern crate collections;

use collections::hashmap::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use inode::{Inode};

mod inode;

pub type RcDirContent = Rc<RefCell<Box<DirectoryContent>>>;
pub type RcInode = Rc<RefCell<Box<Inode>>>;

// File is a thing wrapper around Inodes and Directories. The whole point is to
// provide a layer of indirection. FileHandle's and Directory entries, then,
// point to these guys instead of directly to Inodes/Directories
#[deriving(Clone)]
pub enum File {
  DataFile(RcInode),
  Directory(RcDirContent),
  Empty
}

#[deriving(Clone)]
pub struct FileHandle {
  file: File,
  seek: uint
}

#[deriving(Clone)]
pub struct DirectoryContent {
  pub entries: HashMap<~str, File>
}

pub enum Whence {
  SeekSet,
  SeekCur,
  SeekEnd
}

trait DataFile {
  fn is_data_file(&self) -> bool;
}

impl File {
  pub fn new_dir() -> File {
    let content = box DirectoryContent { entries: HashMap::new() };
    let rc = Rc::new(RefCell::new(content));
    Directory(rc)
  }

  // Don't quite want an inode - want an RC like inode
  pub fn new_data_file(inode: RcInode) -> File {
    DataFile(inode)
  }

  pub fn get_dir_rc<'a>(&'a self) -> &'a RcDirContent {
    match self {
      &Directory(ref rc) => rc,
      _ => fail!("not a directory")
    }
  }

  pub fn get_inode_rc<'a>(&'a self) -> &'a RcInode {
    match self {
      &DataFile(ref rc) => rc,
      _ => fail!("not a directory")
    }
  }
}

impl FileHandle {
  pub fn read(&self, dst: &mut [u8]) -> uint {
    let inode_rc = self.file.get_inode_rc();
    inode_rc.borrow().read(self.seek, dst)
  }

  pub fn write(&mut self, src: &[u8]) -> uint {
    let inode_rc = self.file.get_inode_rc();
    inode_rc.borrow_mut().write(self.seek, src)
  }

  pub fn seek(&mut self, offset: int, whence: Whence) -> uint {
    let inode_rc = self.file.get_inode_rc();

    self.seek = match whence {
      SeekSet => offset as uint,
      SeekCur => (self.seek as int + offset) as uint,
      SeekEnd => (inode_rc.borrow().size() as int + offset) as uint
    };

    self.seek
  }
}

fn main() {}
