use crate::{simple_stdin::simple_stdin, Executor};

pub struct CustomRtldFini;
impl Executor for CustomRtldFini {
    fn execute(&mut self, _i: usize, path: &str) {
        let res = simple_stdin(
            path,
            &[],
            &[("LD_PRELOAD", "target/release/libtransparent_preload.so")],
            Some("Hello, World!"),
        );
        assert!(!res.is_empty());
    }
}
