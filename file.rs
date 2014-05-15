extern crate time;
extern crate collections;

use collections::hashmap::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use inode::*;

mod inode;

pub type RcDirContent = Rc<RefCell<Box<DirectoryContent>>>;
pub type RcInode = Rc<RefCell<Box<Inode>>>;

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

impl File {
  pub fn new_dir() -> File {
    let content = box DirectoryContent { entries: HashMap::new() };
    let rc = Rc::new(RefCell::new(content));
    Directory(rc)
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
