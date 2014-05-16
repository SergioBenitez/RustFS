extern crate time;

use file::{File, Directory};

mod file;
mod inode;

pub trait DirectoryHandle {
  fn is_dir(&self) -> bool;
  fn insert(&mut self, name: ~str, file: Self);
  fn remove(&mut self, name: &~str);
  fn get(&self, name: &~str) -> Option<Self>;
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

  fn remove(&mut self, name: &~str) {
    let rc = self.get_dir_rc();
    let mut content = rc.borrow_mut();
    content.entries.remove(name);
  }

  fn get(&self, name: &~str) -> Option<File> {
    let rc = self.get_dir_rc();
    let content = rc.borrow();
    // TODO: Return none when no file is there.
    match content.entries.find(name) {
      None => None,
      Some(file) => Some(file.clone()) // It's RC
    }
  }
}

// fn main() {
//   use inode::Inode;
//   use std::clone::Clone;
//   use std::rc::Rc;
//   use std::cell::RefCell;

//   let mut dir = File::new_dir(); 
//   println!("{}", dir.is_dir());

//   let filename = "my_file".to_owned();
//   let inode = Rc::new(RefCell::new(box Inode::new()));
//   let file = File::new_data_file(inode.clone());
//   dir.insert(filename, file.clone());

//   // println!("{}: {:?}", filename, file.is_dir());
//   // println!("{}: {:?}", filename, file);
// }
