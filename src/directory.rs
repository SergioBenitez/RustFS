use file::File;
use file::File::Directory;

pub trait DirectoryHandle<'r>: Sized {
  fn is_dir(&self) -> bool;
  fn insert(&mut self, name: &'r str, file: Self);
  fn remove(&mut self, name: &'r str);
  fn get(&self, name: &'r str) -> Option<Self>;
}

impl<'r> DirectoryHandle<'r> for File<'r> {
  fn is_dir(&self) -> bool {
    match self {
      &Directory(_) => true,
      _ => false
    }
  }

  fn insert(&mut self, name: &'r str, file: File<'r>) {
    let rc = self.get_dir_rc();
    let mut content = rc.borrow_mut();
    content.entries.insert(name, file);
  }

  fn remove(&mut self, name: &'r str) {
    let rc = self.get_dir_rc();
    let mut content = rc.borrow_mut();
    content.entries.remove(&name);
  }

  fn get(&self, name: &'r str) -> Option<File<'r>> {
    let rc = self.get_dir_rc();
    let content = rc.borrow();
    match content.entries.get(&name) {
      None => None,
      Some(ref file) => Some((*file).clone()) // It's RC
    }
  }
}
