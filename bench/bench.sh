rm libbench* 2> /dev/null
rm *.o 2> /dev/null

echo "Compling Benchmarking Tool..."
rustc --opt-level=3 ../libbench/lib.rs
mv ../libbench/*.rlib . 2> /dev/null

echo "Compling RustFS..."
rustc --opt-level=3 ../src/proc.rs
mv ../src/*.rlib . 2> /dev/null

echo "Compiling Benchmarks..."
rustc bench.rs --opt-level=3 -L. -o bench.out && ./bench.out
