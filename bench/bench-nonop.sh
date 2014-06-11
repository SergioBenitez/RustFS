rm lib* 2> /dev/null
rm *.o 2> /dev/null
rm *.out 2> /dev/null

echo "Compling Benchmarking Tool..."
rustc -g ../libbench/lib.rs

echo "Compling Slab Allocator Library..."
rustc -g ../libslab/lib.rs

echo "Compling RustFS..."
rustc -g ../src/ary.rs
rustc -g -L. ../src/proc.rs

echo "Compiling Benchmarks..."
rustc bench.rs -g -L. -o bench.out
