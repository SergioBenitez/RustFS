// #[cfg(test)]
mod fs_benchmarks {
  extern crate test;

  use self::test::Bencher;
  use super::super::{Proc, O_CREAT, O_RDWR, FileDescriptor};
  use std::strbuf::StrBuf;

  static NUM: uint = 100;

  fn ceil_div(x: uint, y: uint) -> uint {
    return (x + y - 1) / y;
  }

  fn generate_names(n: uint) -> Vec<StrBuf> {
    let name_length = ceil_div(n, 26);
    let mut name = Vec::from_fn(name_length, |_| '@' as u8);

    Vec::from_fn(n, |i| {
      let next = name.get(i / 26) + 1;
      name.grow_set(i / 26, & ('@' as u8), next);

      let string_result = StrBuf::from_utf8(name.clone());
      match string_result {
        Ok(string) => string,
        Err(_) => fail!("Bad string!")
      }
    })
  }

  fn open_many_c<'a>(p: &mut Proc<'a>, names: &'a Vec<StrBuf>, 
  op: |FileDescriptor, &'a str|) -> Vec<FileDescriptor> {
    Vec::from_fn(names.len(), |i| {
      let filename = names.get(i).as_slice();
      let fd = p.open(filename, O_CREAT | O_RDWR);
      op(fd, filename);
      fd
    })
  }

  fn open_many<'a>(p: &mut Proc<'a>, names: &'a Vec<StrBuf>) -> Vec<FileDescriptor> {
    open_many_c(p, names, |_, _|{ })
  }

  fn close_all(p: &mut Proc, fds: Vec<FileDescriptor>) {
    for fd in fds.iter() {
      p.close(*fd);
    }
  }

  /**
   * Write a two macros to make this easier:
   *
   * bench!({
   *   with_many!(|fd, name| {
   *     p.close(fd);
   *   });
   * });
   *
   * bench! =>
   *   let mut p = Proc::new();
   *   let filesnames = generate_names(NUM);
   *   b.iter(|| user_text);
   *
   * with_many! =>
   *   for i in range(0, NUM) {
   *     let filename = filesnames.get(i).as_slice();
   *     let fd = p.open(filename, O_CREAT | O_RDWR);
   *     user test with fd, name replacement
   *   }
   */

  #[bench]
  fn OC1(b: &mut Bencher) {
    let mut p = Proc::new(); 
    b.iter(|| {
      let fd = p.open("test", O_CREAT);
      p.close(fd);
    });
  }

  #[bench]
  fn OtC(b: &mut Bencher) {
    let mut p = Proc::new(); 
    let filesnames = generate_names(NUM);
    b.iter(|| {
      let fds = open_many(&mut p, &filesnames);
      close_all(&mut p, fds);
    });
  }

  #[bench]
  fn OC(b: &mut Bencher) {
    let mut p = Proc::new(); 
    let filesnames = generate_names(NUM);
    b.iter(|| {
      for i in range(0, NUM) {
        let filename = filesnames.get(i).as_slice();
        let fd = p.open(filename, O_CREAT | O_RDWR);
        p.close(fd);
      }
    });
  }
}
