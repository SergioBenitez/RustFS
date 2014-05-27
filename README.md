RustFS
======

RustFS is a virtual file system written completely in Rust. For examples on how
to use it, see the benchmarks in bench/bench.rs.

Directory Structure
-------------------
* bench/
	* bench.rs # The benchmarks.
	* bench.sh # Script to automatically compile libraries and run benchmarks.

* libbench/lib.rs # The benchmarking library.

* libslab/lib.rs # The slab allocator library.

* src/
  * directory.rs # Insert/Remove/Get directory method implementations.
  * file.rs # FileHandle implementation and structure definitions.
  * inode.rs # Inode structure and implementation.
  * proc.rs # Proc structure (which wrap everything) and implementation.

