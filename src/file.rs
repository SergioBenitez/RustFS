extern crate time;
extern crate collections;

use collections::hashmap::HashMap;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use inode::{Inode};
use directory::DirectoryHandle;

type RcDirContent = Rc<RefCell<Box<DirectoryContent>>>;
type RcInode = Rc<RefCell<Box<Inode>>>;

// File is a thing wrapper around Inodes and Directories. The whole point is to
// provide a layer of indirection. FileHandle's and Directory entries, then,
// point to these guys instead of directly to Inodes/Directories
#[deriving(Clone)]
pub enum File {
  DataFile(RcInode),
  Directory(RcDirContent),
  EmptyFile
}

#[deriving(Clone)]
pub struct FileHandle {
  file: File,
  seek: Cell<uint>
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
  pub fn new_dir(parent: Option<File>) -> File {
    let content = box DirectoryContent { entries: HashMap::new() };
    let rc = Rc::new(RefCell::new(content));
    let dir = Directory(rc);

    // Note that dir is RCd, so this is cheap
    // Used to borrow dir and mut_dir at "same time"
    // RefCell makes sure we're not doing anything wrong
    let mut mut_dir = dir.clone();

    // Setting up "." and ".."
    mut_dir.insert(".".to_owned(), dir.clone());
    match parent {
      None => mut_dir.insert("..".to_owned(), dir.clone()),
      Some(f) => mut_dir.insert("..".to_owned(), f)
    }

    dir
  }

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
  // Probably not the right type.
  pub fn new(file: File) -> FileHandle {
    FileHandle {
      file: file,
      seek: Cell::new(0)
    }
  }

  pub fn read(&self, dst: &mut [u8]) -> uint {
    let offset = self.seek.get();
    let inode_rc = self.file.get_inode_rc();
    let changed = inode_rc.borrow().read(offset, dst);
    self.seek.set(offset + changed);
    changed
  }

  pub fn write(&mut self, src: &[u8]) -> uint {
    let offset = self.seek.get();
    let inode_rc = self.file.get_inode_rc();
    let changed = inode_rc.borrow_mut().write(offset, src);
    self.seek.set(offset + changed);
    changed
  }

  pub fn seek(&mut self, offset: int, whence: Whence) -> uint {
    let inode_rc = self.file.get_inode_rc();

    let seek = self.seek.get();
    let new_seek = match whence {
      SeekSet => offset as uint,
      SeekCur => (seek as int + offset) as uint,
      SeekEnd => (inode_rc.borrow().size() as int + offset) as uint
    };

    self.seek.set(new_seek);
    new_seek
  }
}
