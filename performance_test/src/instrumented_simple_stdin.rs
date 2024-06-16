use crate::{simple_stdin::simple_stdin, Executor};

struct InstrumentedGNUSimpleStdin;
impl Executor for InstrumentedGNUSimpleStdin {
    fn execute(&mut self, i: usize) {
        simple_stdin(
            &[],
            &[],
            Some("Hello, World!"),
            Some("../fuzzer/target/GNU_coreutils/src/base64"),
        )
    }
}
