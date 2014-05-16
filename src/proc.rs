extern crate rand;
extern crate time;
extern crate collections;

pub use file::Whence;
use file::{File, EmptyFile, DataFile, Directory, FileHandle};
use inode::Inode;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use collections::HashMap;
use directory::DirectoryHandle;

mod directory;
mod file;
mod inode;

pub type FileDescriptor = int;

static O_RDONLY: u32 =   (1 << 0);
static O_WRONLY: u32 =   (1 << 1);
static O_RDWR: u32 =     (1 << 2);
static O_NONBLOCK: u32 = (1 << 3);
static O_APPEND: u32 =   (1 << 4);
static O_CREAT: u32 =    (1 << 5);

pub struct Proc {
  cwd: File,
  fd_table: HashMap<FileDescriptor, FileHandle>,
  last_fd: Cell<int>
}

impl Proc {
  pub fn new() -> Proc {
    Proc {
      cwd: File::new_dir(),
      fd_table: HashMap::new(),
      last_fd: Cell::new(2)
    }
  }

  fn get_fd(&self) -> FileDescriptor {
    let fd = self.last_fd.get() + 1;
    self.last_fd.set(fd);
    fd
  }

  pub fn open(&mut self, path: ~str, flags: u32) -> FileDescriptor {
    println!("Lookup: {}", path);

    let lookup = self.cwd.get(&path);
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
        let fd = self.get_fd();
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
  }

  pub fn unlink(&mut self, path: &~str) {
    self.cwd.remove(path);
  }
}

#[cfg(test)]
mod proc_tests {
  extern crate test;

  use super::{Proc, O_RDWR, O_CREAT};
  use file::{Whence, SeekSet};
  use rand::random;

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
    let filename = "first_file".to_owned();

    let fd = p.open(filename.clone(), O_RDWR | O_CREAT);
    p.write(fd, data.as_slice());
    p.seek(fd, 0, SeekSet);
    p.read(fd, buf);
    
    assert_eq_buf(data.as_slice(), buf);

    let fd2 = p.open(filename.clone(), O_RDWR);
    let mut buf2 = [0u8, ..size];
    p.read(fd2, buf2);

    assert_eq_buf(data.as_slice(), buf2);

    p.close(fd);
    p.close(fd2);

    let fd3 = p.open(filename.clone(), O_RDWR);
    let mut buf3 = [0u8, ..size];
    p.read(fd3, buf3);

    assert_eq_buf(data.as_slice(), buf3);
    p.close(fd3);

    p.unlink(&filename);

    // Verify file is no longer there
    // TODO: Verify that the data was indeed deallocated, but it's unclear what
    // the easiest way to do that it. In any case, it's important that it's done
    let fd4 = p.open(filename, O_RDWR);
    assert_eq!(fd4, -2);
  }
}

fn main() {}
