use std::time::{Duration, Instant};

use performance_test::*;

#[allow(dead_code)]
static UNINSTRUMENTED_UUTILS: &str = "./uutils_coreutils/target/release/base64";
#[allow(dead_code)]
static UNINSTRUMENTED_GNU: &str = "./GNU_coreutils/src/base64";
#[allow(dead_code)]
static INSTRUMENTED_GNU: &str = "../fuzzer/target/GNU_coreutils/src/base64";
#[allow(dead_code)]
static INSTRUMENTED_UUTILS: &str = "../fuzzer/target/uutils_coreutils/target/release/base64";
static ITERS: usize = 10_000;

fn main() {
    (0..10).for_each(|_| println!("{:?}", run_test(SimpleStdin, UNINSTRUMENTED_GNU)));
}

#[allow(dead_code)]
fn run_test<E: Executor + Sync>(mut e: E, path: &str) -> Duration {
    e.setup();
    let start = Instant::now();
    (0..ITERS).for_each(|i| e.execute(i, path));
    let res = start.elapsed();
    e.teardown();
    res
}
