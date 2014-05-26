
// #[cfg(test)]
mod fs_benchmarks {
  extern crate test;

  use self::test::Bencher;
  use super::super::{Proc, O_CREAT, O_RDWR, FileDescriptor};
  use std::strbuf::StrBuf;

  static NUM: uint = 100;

  macro_rules! bench(
    (|$p:ident, $filenames:ident| $task:stmt) => ({
      let mut $p = Proc::new();
      let $filenames = generate_names(NUM);
      b.iter(|| { $task });
    });
  )

  macro_rules! bench_many(
    (|$p:ident, $fd:ident, $filename:ident| $op:stmt) => ({
      let mut $p = Proc::new();
      let filenames = generate_names(NUM);
      b.iter(|| { 
        for i in range(0, NUM) {
          let $filename = filenames.get(i).as_slice();
          let $fd = $p.open($filename, O_CREAT | O_RDWR);
          $op
        }
      });
    });
  )

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

  fn open_many<'a>(p: &mut Proc<'a>, names: &'a Vec<StrBuf>) -> Vec<FileDescriptor> {
    Vec::from_fn(names.len(), |i| {
      let filename = names.get(i).as_slice();
      let fd = p.open(filename, O_CREAT | O_RDWR);
      fd
    })
  }

  fn close_all(p: &mut Proc, fds: Vec<FileDescriptor>) {
    for fd in fds.iter() {
      p.close(*fd);
    }
  }

  fn unlink_all<'a>(p: &mut Proc<'a>, names: &'a Vec<StrBuf>) {
    for filename in names.iter() {
      p.unlink(filename.as_slice());
    }
  }

  #[bench]
  fn OC1(b: &mut Bencher) {
    bench!(|p, _names| {
      let fd = p.open("test", O_CREAT);
      p.close(fd);
    });
  }

  #[bench]
  fn OtC(b: &mut Bencher) {
    bench!(|p, filenames| {
      let fds = open_many(&mut p, &filenames);
      close_all(&mut p, fds);
    });
  }

  #[bench]
  fn OC(b: &mut Bencher) {
    bench_many!(|p, fd, filename| {
      p.close(fd);
    });
  }

  #[bench]
  fn OtCtU(b: &mut Bencher) {
    bench!(|p, filenames| {
      let fds = open_many(&mut p, &filenames);
      close_all(&mut p, fds);
      unlink_all(&mut p, &filenames);
    });
  }

  #[bench]
  fn OCU(b: &mut Bencher) {
    bench_many!(|p, fd, filename| {
      p.close(fd);
      p.unlink(filename);
    });
  }
}
