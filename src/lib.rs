extern crate time;
#[macro_use]
extern crate bitflags;

mod directory;
mod file;
mod inode;

use file::{File, FileHandle};
use file::File::{EmptyFile, DataFile, Directory};
use std::rc::Rc;
use std::cell::{RefCell};
use std::collections::HashMap;
use directory::DirectoryHandle;
pub use file::Whence;
pub use inode::Inode;

pub type FileDescriptor = isize;

bitflags!{
    pub struct FileFlags: u32 {
        const O_RDONLY =   0b00000001;
        const O_WRONLY =   0b00000010;
        const O_RDWR =     0b00000100;
        const O_NONBLOCK = 0b00001000;
        const O_APPEND =   0b00010000;
        const O_CREAT =    0b00100000;
    }
}

pub struct Proc<'r> {
  cwd: File<'r>,
  fd_table: HashMap<FileDescriptor, FileHandle<'r>>,
  fds: Vec<FileDescriptor>
}

impl<'r> Proc<'r> {
  pub fn new() -> Proc<'r> {
    Proc {
      cwd: File::new_dir(None),
      fd_table: HashMap::new(),
      fds: (0..(256 - 2)).map(|i| 256 - i).collect(),
    }
  }

  #[inline(always)]
  fn extract_fd(fd_opt: &Option<FileDescriptor>) -> FileDescriptor {
    match fd_opt {
      &Some(fd) => fd,
      &None => panic!("Error in FD allocation.")
    }
  }

  pub fn open(&mut self, path: &'r str, flags: FileFlags) -> FileDescriptor {
    let lookup = self.cwd.get(path);
    let file = match lookup {
      Some(f) => f,
      None => {
        if (flags & FileFlags::O_CREAT) == FileFlags::O_CREAT {
          // FIXME: Fetch from allocator
          let rcinode = Rc::new(RefCell::new(Box::new(Inode::new())));
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

  pub fn read(&self, fd: FileDescriptor, dst: &mut [u8]) -> usize {
    let handle = self.fd_table.get(&fd).expect("fd does not exist");
    handle.read(dst)
  }

  pub fn write(&mut self, fd: FileDescriptor, src: &[u8]) -> usize {
    let handle = self.fd_table.get_mut(&fd).expect("fd does not exist");
    handle.write(src)
  }

  pub fn seek(&mut self, fd: FileDescriptor, o: isize, whence: Whence) -> usize {
    let handle = self.fd_table.get_mut(&fd).expect("fd does not exist");
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
  // extern crate test;
  extern crate rand;

  use super::{Proc, FileFlags};
  use file::Whence::SeekSet;
  use inode::Inode;
  use self::rand::random;

  static mut test_inode_drop: bool = false;

  impl Drop for Inode {
    fn drop(&mut self) {
      unsafe {
        if test_inode_drop {
          test_inode_drop = false;
          panic!("Dropping.");
        } else {
          println!("Dropping, but no flag.");
        }
      }
    }
  }

  fn rand_array(size: usize) -> Vec<u8> {
    (0..size).map(|_| random::<u8>()).collect()
  }

  fn assert_eq_buf(first: &[u8], second: &[u8]) {
    assert_eq!(first.len(), second.len());

    for i in 0..first.len() {
      assert_eq!(first[i], second[i]);
    }
  }

  #[test]
  fn simple_test() {
    const SIZE: usize = 4096 * 8 + 3434;
    let mut p = Proc::new();
    let data = rand_array(SIZE);
    let mut buf = [0u8; SIZE];
    let filename = "first_file";

    let fd = p.open(filename, FileFlags::O_RDWR | FileFlags::O_CREAT);
    p.write(fd, &data);
    p.seek(fd, 0, SeekSet);
    p.read(fd, &mut buf);

    assert_eq_buf(&data, &buf);

    let fd2 = p.open(filename, FileFlags::O_RDWR);
    let mut buf2 = [0u8; SIZE];
    p.read(fd2, &mut buf2);

    assert_eq_buf(&data, &buf2);

    p.close(fd);
    p.close(fd2);

    let fd3 = p.open(filename, FileFlags::O_RDWR);
    let mut buf3 = [0u8; SIZE];
    p.read(fd3, &mut buf3);

    assert_eq_buf(&data, &buf3);
    p.close(fd3);

    p.unlink(filename);

    let fd4 = p.open(filename, FileFlags::O_RDWR);
    assert_eq!(fd4, -2);
  }

  #[test]
  #[should_panic]
  fn test_proc_drop_inode_dealloc() {
    // Variable is used to make sure that the Drop implemented is only valid for
    // tests that set that test_inode_drop global variable to true.
    unsafe { test_inode_drop = true; }

    const SIZE: usize = 4096 * 3 + 3498;
    let mut p = Proc::new();
    let mut data = rand_array(SIZE);

    let fd = p.open("file", FileFlags::O_RDWR | FileFlags::O_CREAT);
    p.write(fd, &mut data);
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
  #[should_panic]
  fn test_inode_dealloc() {
    // Make sure flag is set to detect drop.
    unsafe { test_inode_drop = true; }

    const SIZE: usize = 4096 * 3 + 3498;
    let mut p = Proc::new();
    let mut data = rand_array(SIZE);
    let mut buf = [0u8; SIZE];
    let filename = "first_file";

    let fd = p.open(filename, FileFlags::O_RDWR | FileFlags::O_CREAT);
    p.write(fd, &mut data);
    p.seek(fd, 0, SeekSet);
    p.read(fd, &mut buf);

    assert_eq_buf(&data, &buf);

    // close + unlink should remove both references to inode, dropping it,
    // causing a failure
    p.close(fd);
    p.unlink(filename);

    // If inode is not being dropped properly, ie, on the unlink call this will
    // cause a double failure: once for panic! call, and once when then the Inode
    // is dropped since the Proc structure will be dropped.
    //
    // To test that RC is working properly, make sure that a double failure
    // occurs when either the close or unlink calls above are commented out.
    panic!("Inode not dropped!");
  }

  #[test]
  fn test_max_singly_file_size() {
    const SIZE: usize = 4096 * 256;
    let mut p = Proc::new();
    let mut data = rand_array(SIZE);
    let mut buf = [0u8; SIZE];
    let filename = "first_file";

    let fd = p.open(filename, FileFlags::O_RDWR | FileFlags::O_CREAT);
    p.write(fd, &mut data);
    p.seek(fd, 0, SeekSet);
    p.read(fd, &mut buf);

    assert_eq_buf(&data, &buf);

    p.close(fd);
    p.unlink(filename);

    let fd4 = p.open(filename, FileFlags::O_RDWR);
    assert_eq!(fd4, -2);
  }

  #[test]
  fn test_max_file_size() {
    const SIZE: usize = 2 * 4096 * 256;
    let mut p = Proc::new();
    let mut data1 = rand_array(SIZE);
    let mut data2 = rand_array(SIZE);
    let mut buf = vec![0; SIZE];
    let filename = "first_file";

    let fd = p.open(filename, FileFlags::O_RDWR | FileFlags::O_CREAT);
    p.write(fd, &mut data1);
    p.seek(fd, 4096 * 257 * 256 - SIZE as isize, SeekSet);
    p.write(fd, &mut data2);

    p.seek(fd, 0, SeekSet);
    p.read(fd, &mut buf);
    assert_eq_buf(&data1, &buf);

    p.seek(fd, 4096 * 257 * 256 - SIZE as isize, SeekSet);
    p.read(fd, &mut buf);
    assert_eq_buf(&data2, &buf);
  }

  #[test]
  #[should_panic]
  fn test_morethan_max_file_size() {
    const SIZE: usize = 2 * 4096 * 256;
    let mut p = Proc::new();
    let mut data = rand_array(SIZE);
    let filename = "first_file";

    let fd = p.open(filename, FileFlags::O_RDWR | FileFlags::O_CREAT);
    p.write(fd, &mut data);
    p.seek(fd, 4096 * 257 * 256 + 1 - SIZE as isize, SeekSet);
    p.write(fd, &mut data);
  }
}
