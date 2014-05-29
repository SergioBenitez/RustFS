rm lib* 2> /dev/null
rm *.o 2> /dev/null
rm *.out 2> /dev/null

echo "Compling Benchmarking Tool..."
rustc --opt-level=3 ../libbench/lib.rs

echo "Compling Slab Allocator Library..."
rustc --opt-level=3 ../libslab/lib.rs

echo "Compling RustFS..."
rustc --opt-level=3 ../src/ary.rs
rustc -L. --opt-level=3 ../src/proc.rs

echo "Compiling Benchmarks..."
rustc bench.rs --opt-level=3 -L. -o bench.out && ./bench.out
