rustc --opt-level=3 ../libslab/lib.rs
rustc --opt-level=3 ary.rs
RUST_TEST_TASKS=1 rustc proc.rs -L. --test -o rustfs.out
RUST_TEST_TASKS=1 ./rustfs.out
