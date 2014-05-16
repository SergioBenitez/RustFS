#![feature(globs)]

extern crate time;
extern crate collections;

use file::{File, Directory};

mod file;
mod inode;

trait DirectoryHandle {
  fn is_dir(&self) -> bool;
  fn insert(&mut self, name: ~str, file: Self);
  fn get(&self, name: ~str) -> Self;
}

impl DirectoryHandle for File {
  fn is_dir(&self) -> bool {
    match self {
      &Directory(_) => true,
      _ => false
    }
  }

  fn insert(&mut self, name: ~str, file: File) {
    let rc = self.get_dir_rc();
    let mut content = rc.borrow_mut();
    content.entries.insert(name, file); // RC
  }

  fn get(&self, name: ~str) -> File {
    let rc = self.get_dir_rc();
    let content = rc.borrow();
    content.entries.get(&name).clone() // It's RC
  }
}

fn main() {
  use file::Empty;
  use inode::Inode;
  use std::clone::Clone;
  use std::rc::Rc;
  use std::cell::RefCell;

  let mut dir = File::new_dir(); 
  println!("{}", dir.is_dir());

  let filename = "my_file".to_owned();
  let inode = Rc::new(RefCell::new(box Inode::new()));
  let file = File::new_data_file(inode.clone());
  dir.insert("my_file".to_owned(), file.clone());

  println!("{}: {:?}", filename, file.is_dir());
  println!("{}: {:?}", filename, file);
}
