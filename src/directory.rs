use file::{File, Directory};

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
    content.entries.insert(name, file);
  }

  fn remove(&mut self, name: &~str) {
    let rc = self.get_dir_rc();
    let mut content = rc.borrow_mut();
    content.entries.remove(name);
  }

  fn get(&self, name: &~str) -> Option<File> {
    let rc = self.get_dir_rc();
    let content = rc.borrow();
    match content.entries.find(name) {
      None => None,
      Some(ref file) => Some((*file).clone()) // It's RC
    }
  }
}
