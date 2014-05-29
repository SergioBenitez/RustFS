rustc --opt-level=3 ../libslab/lib.rs
RUST_TEST_TASKS=1 rustc proc.rs -L. --test -o rustfs.out
RUST_TEST_TASKS=1 ./rustfs.out
