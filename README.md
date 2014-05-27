RustFS
======

RustFS is a virtual file system written completely in Rust.

Usage
-----
First, compile the RustFS library:

	rustc --opt-level=3 src/proc.rs
	
Place the resulting librustfs file into the directory where you progam lives.

Then, import the crate into your project and bring types into the namespace:
	
	extern crate rustfs;
	
	use rustfs::{Proc, O_CREAT, O_RDWR};
	
Finally, use `Proc::new()` to create a new `Proc`. Call `open` / `close` / `seek` / `read` / `write` on it:

	let mut p = Proc::new();
	
	# Let's write `data` to a new file named "file".
	let data = ... some data ...;
    let fd = p.open("file", O_CREAT | O_RDWR);
    p.write(fd, data);
    p.close(fd);
    
    # Let's read back that data to a buffer named `buf` of the correct size.
    let mut buf = [0u8, ..size];
    let fd = p.open("file", O_RDWR);
    p.read(fd, buf);
    p.close(fd);
    
    # All done. Unlink.
	p.unlink("file");

For more examples on how to use RustFS, see the benchmarks in bench/bench.rs and tests in src/proc.rs.

Directory Structure
-------------------
* bench/
  * bench.rs _The benchmarks._
  * bench.sh _Script to automatically compile libraries and run benchmarks._

* libbench/lib.rs _The benchmarking library._

* libslab/lib.rs _The slab allocator library._

* src/
  * directory.rs _Insert/Remove/Get directory method implementations._
  * file.rs _FileHandle implementation and structure definitions._
  * inode.rs _Inode structure and implementation._
  * proc.rs _Proc structure (which wraps everything) and implementation._