# Coreutils Coverage

This is an example of how to perform coverage-guided fuzzing on a binary using a `CommandConfigurator`/`CommandExecutor`. Specifically, it targets coreutils, which adds the following requirements:
- Coreutils use a complex build system (Autotools), and this approach does not require any changes to the build system (nor the source code for that matter)
- The only changes are a different compiler, and different compilation flags, all of which can be passed as command line arguments and/or environment variables (see [Makefile.toml](./Makefile.toml))
- The binary under test calls `exit` — this is why a `CommandExecutor` is required (since this would quit the fuzzer otherwise)
- Inputs are a combination of command line arguments, `stdin`, and even files
- It further allows analyzing all output of the binary under test

## Usage

This project uses [cargo-make](https://sagiegurari.github.io/cargo-make/). Use `cargo make fuzzer` to build all necessary components or invoke the fuzzer directly with `cargo make run`.

## How Coverage is transmitted
1. Coreutils are compiled using clang's `-fsanitize-coverage=trace-pc-guard` (similar to LibAFL_target, and may in the future be unified)
   1. First, a custom coverage collection handler ([coverage.c](./coverage.c)) is compiled to an object file.
   2. Then, this file is passed to the compiler and linked instead of the default handler.
2. Before starting the actual fuzzing, the fuzzer performs the following steps:
   1. First, the fuzzer loads the binary under test with an overridden main function (using the shared library created in [get_guard_num](./get_guard_num/)), which interfaces with the linked custom coverage gathering functionality to retrieve the number of guards. This number is then passed back to the fuzzer without calling the main function of the binary under test.
   2. Then, the fuzzer allocates a shared memory region with the retrieved size and sets it up to be accessible from the child.
3. Then, the actual fuzzing starts:
   1. The binary is called with a reference to the shared memory region.
   2. The binary is once again wrapped with a shared library. This wrapper ([setup_guard_redirection](./setup_guard_redirection/)) first parses the shared memory reference. It then removes any reference from `argc`/`argv` and calls the main function of the binary under test.
   3. During the teardown phase, the wrapper once again intersects the usual program flow to copy the coverage data collected during execution to the shared memory region. It then continues teardown.
   4. This data is then accessible from the fuzzer and can be used to guide fuzzing.