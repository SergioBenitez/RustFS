#![feature(macro_rules)]

extern crate rand;
extern crate time;
extern crate collections;

pub use file::Whence;
use file::{File, EmptyFile, DataFile, Directory, FileHandle};
use inode::Inode;
use std::rc::Rc;
use std::cell::{RefCell};
use collections::HashMap;
use directory::DirectoryHandle;

mod directory;
mod file;
mod inode;
mod bench;

pub type FileDescriptor = int;

pub static O_RDONLY: u32 =   (1 << 0);
pub static O_WRONLY: u32 =   (1 << 1);
pub static O_RDWR: u32 =     (1 << 2);
pub static O_NONBLOCK: u32 = (1 << 3);
pub static O_APPEND: u32 =   (1 << 4);
pub static O_CREAT: u32 =    (1 << 5);

pub struct Proc<'r> {
  cwd: File<'r>,
  fd_table: HashMap<FileDescriptor, FileHandle<'r>>,
  fds: Vec<FileDescriptor>
}

impl<'r> Proc<'r> {
  pub fn new() -> Proc {
    Proc {
      cwd: File::new_dir(None),
      fd_table: HashMap::new(),
      fds: Vec::from_fn(256 - 2, |i| { 256i - (i as int) })
    }
  }
  
  #[inline(always)]
  fn extract_fd(fd_opt: &Option<FileDescriptor>) -> FileDescriptor {
    match fd_opt {
      &Some(fd) => fd,
      &None => fail!("Error in FD allocation.")
    }
  }

  pub fn open(&mut self, path: &'r str, flags: u32) -> FileDescriptor {
    let lookup = self.cwd.get(path);
    let file = match lookup {
      Some(f) => f,
      None => {
        if (flags & O_CREAT) != 0 {
          // FIXME: Fetch from allocator
          let rcinode = Rc::new(RefCell::new(box Inode::new()));
          let file = File::new_data_file(rcinode);
          self.cwd.insert(path, file.clone());
          file
        } else {
          EmptyFile
        }
      }
    };

    match file {
      DataFile(_) => {
        let fd = Proc::extract_fd(&self.fds.pop());
        let handle = FileHandle::new(file);
        self.fd_table.insert(fd, handle);
        fd
      }
      Directory(_) => -1,
      EmptyFile => -2,
    }
  }

  pub fn read(&self, fd: FileDescriptor, dst: &mut [u8]) -> uint {
    let handle = self.fd_table.get(&fd);
    handle.read(dst)
  }

  pub fn write(&mut self, fd: FileDescriptor, src: &[u8]) -> uint {
    let handle = self.fd_table.get_mut(&fd);
    handle.write(src)
  }

  pub fn seek(&mut self, fd: FileDescriptor, o: int, whence: Whence) -> uint {
    let handle = self.fd_table.get_mut(&fd);
    handle.seek(o, whence)
  }

  pub fn close(&mut self, fd: FileDescriptor) {
    self.fd_table.remove(&fd);
    self.fds.push(fd);
  }

  pub fn unlink(&mut self, path: &'r str) {
    self.cwd.remove(path);
  }
}

#[cfg(test)]
mod proc_tests {
  extern crate test;

  use super::{Proc, O_RDWR, O_CREAT};
  use file::{SeekSet};
  use inode::Inode;
  use rand::random;

  static mut test_inode_drop: bool = false;

  fn rand_array(size: uint) -> Vec<u8> {
    Vec::from_fn(size, |_| {
      random::<u8>()
    })
  }

  fn assert_eq_buf(first: &[u8], second: &[u8]) {
    assert_eq!(first.len(), second.len());

    for i in range(0, first.len()) {
      assert_eq!(first[i], second[i]);
    }
  }

  #[test]
  fn simple_test() {
    static size: uint = 4096 * 8 + 3434;
    let mut p = Proc::new();
    let data = rand_array(size);
    let mut buf = [0u8, ..size];
    let filename = "first_file";

    let fd = p.open(filename, O_RDWR | O_CREAT);
    p.write(fd, data.as_slice());
    p.seek(fd, 0, SeekSet);
    p.read(fd, buf);
    
    assert_eq_buf(data.as_slice(), buf);

    let fd2 = p.open(filename, O_RDWR);
    let mut buf2 = [0u8, ..size];
    p.read(fd2, buf2);

    assert_eq_buf(data.as_slice(), buf2);

    p.close(fd);
    p.close(fd2);

    let fd3 = p.open(filename, O_RDWR);
    let mut buf3 = [0u8, ..size];
    p.read(fd3, buf3);

    assert_eq_buf(data.as_slice(), buf3);
    p.close(fd3);

    p.unlink(filename);

    let fd4 = p.open(filename, O_RDWR);
    assert_eq!(fd4, -2);
  }

  /**
   * This function makes sure that on unlink, the inode's data structure is
   * indeed dropped. This means that a few things have gone right:
   *
   * 1) The FileHandle was dropped. If it wasn't, it would hold a reference to
   *    the file and so the file wouldn't be dropped. This should happen on
   *    close.
   * 2) The File, containing the Inode, was dropped. This should happen on
   *    unlink.
   */
  #[test]
  #[should_fail]
  fn test_inode_dealloc() {
    // Variable is used to make sure that the Drop implemented is only valid for
    // tests that set that test_inode_drop global variable to true.
    unsafe { test_inode_drop = true; }

    impl Drop for Inode {
      fn drop(&mut self) {
        unsafe { 
          if test_inode_drop {
            test_inode_drop = false;
            fail!("Dropping.");
          }
        }
      }
    }

    static size: uint = 4096;
    let mut p = Proc::new();
    let data = rand_array(size);
    let mut buf = [0u8, ..size];
    let filename = "first_file";

    let fd = p.open(filename, O_RDWR | O_CREAT);
    p.write(fd, data.as_slice());
    p.seek(fd, 0, SeekSet);
    p.read(fd, buf);

    assert_eq_buf(data.as_slice(), buf);

    p.close(fd);
    p.unlink(filename);
    
    // If inode is not being dropped properly, ie, on the unlink call this will
    // cause a double failure: once for fail! call, and once when then the Inode
    // is dropped since the Proc structure will be dropped.
    //
    // To test that RC is working properly, make sure that a double failure
    // occurs when either the close or unlink calls above are commented out.
    fail!("Inode not dropped!");
  }

  #[test]
  fn test_max_file_size() {
    static size: uint = 4096 * 256;
    let mut p = Proc::new();
    let data = rand_array(size);
    let mut buf = [0u8, ..size];
    let filename = "first_file";

    let fd = p.open(filename, O_RDWR | O_CREAT);
    p.write(fd, data.as_slice());
    p.seek(fd, 0, SeekSet);
    p.read(fd, buf);
    
    assert_eq_buf(data.as_slice(), buf);

    p.close(fd);
    p.unlink(filename);

    let fd4 = p.open(filename, O_RDWR);
    assert_eq!(fd4, -2);
  }

  #[test]
  #[should_fail]
  fn test_morethan_max_file_size() {
    static size: uint = 4096 * 256 + 1;
    let mut p = Proc::new();
    let data = rand_array(size);
    let filename = "first_file";

    let fd = p.open(filename, O_RDWR | O_CREAT);
    p.write(fd, data.as_slice());
  }
}
