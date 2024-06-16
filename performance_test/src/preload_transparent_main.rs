use crate::{simple_stdin::simple_stdin, Executor};

pub struct PreloadTransparentMain;
impl Executor for PreloadTransparentMain {
    fn execute(&mut self, _i: usize, path: &str) {
        let res = simple_stdin(
            path,
            &[],
            &[("LD_PRELOAD", "target/release/librtld_fini_preload.so")],
            Some("Hello, World!"),
        );
        assert!(!res.is_empty());
    }
}
