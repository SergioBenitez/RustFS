extern crate time;
extern crate collections;

use file::{File, FileHandle};
use collections::HashMap;
use directory::DirectoryHandle;

mod directory;
mod file;
mod inode;

pub type FileDescriptor = int;

static O_RDONLY: u32 =  (1 << 0);
static O_WRONLY: u32 =   (1 << 1);
static O_RDWR: u32 =     (1 << 2);
static O_NONBLOCK: u32 = (1 << 3);
static O_APPEND: u32 =   (1 << 4);
static O_CREAT: u32 =    (1 << 5);

pub struct Proc {
  cwd: File,
  fd_table: HashMap<FileDescriptor, FileHandle>
}

impl Proc {
  pub fn new() -> Proc {
    Proc {
      cwd: File::new_dir(),
      fd_table: HashMap::new()
    }
  }

  pub fn open(&mut self, path: ~str, flags: u32) -> FileDescriptor {
    let lookup = self.cwd.get(path);
    let found = match lookup {
      Some(_) => true,
      None => false
    };

    if !found && (flags & O_CREAT) != 0 {
      // create a new file
      return 0;
    }

    // simply create a handle to the file
    1
  }
}

#[cfg(test)]
mod tests {
  extern crate test;

  #[test]
  fn simple_test() {
    assert_eq!(1, 1);
  }
}

fn main() {
  let p = Proc::new();
  println!("{:?}", p);
}
