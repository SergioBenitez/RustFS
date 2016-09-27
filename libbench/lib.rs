#![feature(asm)]

extern crate time;

use std::cmp;
use time::precise_time_ns;

pub struct Benchmarker {
  iterations: u64,
  ns_start: u64,
  ns_end: u64,
}

pub type IterationCount = u64;
pub type BenchTimeNS = u64;
pub type BenchResults = (IterationCount, BenchTimeNS);

fn black_box<T>(dummy: T) {
  // we need to "use" the argument in some way LLVM can't introspect.
  unsafe {asm!("" : : "r"(&dummy))}
}

impl Benchmarker {
  pub fn new() -> Benchmarker {
    Benchmarker {
      iterations: 0,
      ns_start: 0,
      ns_end: 0
    }
  }

  pub fn run<F: FnMut()>(&mut self, mut f: F) -> BenchTimeNS {
    let k = self.iterations;
    self.ns_start = precise_time_ns();
    for _ in 0..k {
      black_box(f());
    }
    self.ns_end = precise_time_ns();
    self.ns_end - self.ns_start
  }

  fn ns_per_iter(&self) -> BenchTimeNS {
    (self.ns_end - self.ns_start) / self.iterations
  }

  pub fn bench_n<F: FnMut(&mut Benchmarker)>(&mut self, n: u64, mut f: F) {
    self.iterations = n;
    f(self); // f will call b.run internally
  }

  pub fn bench<F: FnMut(&mut Benchmarker)>(&mut self, mut f: F, min_time: u64) -> BenchResults {
    // min_time is in ms, convert to ns. start with 1 iteration
    let min_time = min_time * 1_000_000;
    let mut n: u64 = 1;

    // Keep trying to get enough iterations so as take `min_time`
    loop {
      // run for n iterations
      self.bench_n(n, |x| f(x));

      // If we've done enough, we're done
      let elapsed = self.ns_end - self.ns_start;
      if elapsed == 0 { panic!("Must call run in benchmark function."); }
      if elapsed >= min_time { break }

      // Otherwise, adjust the number of iterations and try again
      let new_n = n * (min_time / elapsed);
      n = cmp::max((new_n as f64 * 1.2) as u64, n + 1);
    }

    (self.iterations, self.ns_per_iter())
  }

  pub fn print_results(&self, name: &str, results: BenchResults) {
    let (iter, ns_per_iter) = results;
    println!("{:10}: {:12} ns/iter ({} it.)", name, ns_per_iter, iter);
  }
}

// Benchmark `f` for at least `time` ms
pub fn benchmark<F: FnMut(&mut Benchmarker)>(name: &str, f: F, time: u64) -> BenchResults {
  let mut bench = Benchmarker::new();
  let results = bench.bench(f, time);
  bench.print_results(name, results);
  results
}

// #[cfg(bench)]
// fn main() {
//   fn insert_1000() {
//     let mut v = Vec::new();
//     for i in range(0, 1000) {
//       v.push(i);
//     }
//   }

//   let (iter, _) = benchmark("insert_1000", insert_1000, 1);
//   assert!(iter > 10 && iter < 30);
// }
