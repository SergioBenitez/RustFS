rm lib* 2> /dev/null
rm *.o 2> /dev/null
rm *.out 2> /dev/null

echo "Compling Benchmarking Tool..."
rustc ../libbench/lib.rs

echo "Compling Slab Allocator Library..."
rustc ../libslab/lib.rs

echo "Compling RustFS..."
rustc -L. ../src/proc.rs

echo "Compiling Benchmarks..."
rustc bench.rs -g -L. -o bench.out -C link-args="-lprofiler"
CPUPROFILE=/tmp/cpuprofile ./bench.out
