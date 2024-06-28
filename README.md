# Differential Fuzzing on coreutils

## Report

Read the report [here](./report/out/index.pdf). Its artifacts are in the `report` subdirectory. The report is licensed under [CC BY-NC-ND 4.0](https://creativecommons.org/licenses/by-nc-nd/4.0/).

## Performance Tests

Code to perform performance tests outside of the fuzzer and their results can be found in [performance_test](./performance_test/).

## Fuzzer

This project is written in rust/cargo (make sure you have a [current version installed](https://doc.rust-lang.org/cargo/getting-started/installation.html)) and uses [cargo make](https://sagiegurari.github.io/cargo-make/) to document and automate the build steps.

### Clone
Clone the repository using
```bash
git clone --recurse-submodules https://github.com/riesentoaster/coreutils-differential-fuzzing.git
```

### Build and Run

To build all the necessary artifacts
- Navigate to the fuzzer directory: `cd coreutils-differential-fuzzing/fuzzer`
- Run the fuzzer in its full differential mode using `cargo make run`
  - The build process may take a few minutes since it contains multiple helper binaries and both GNU's and coreutils' version of coreutils.
  - Check out the options using `cargo make run --help`, you may want to use some like `cargo make run --cores 0-16`

Other targets include:
- `cargo make fuzzer` to only build the binaries without starting the fuzzer
- `cargo make fuzzer_gnu` to only run on GNU's version of coreutils (without differential fuzzing)
- `cargo make fuzzer_uutils` to only run uutils' version
- `cargo make run` to directly run the fuzzer (resp. `cargo make run_gnu`/`cargo make run_uutils` to directly run the fuzzer on one implementation only)
- Check [`Makefile.toml`](./fuzzer/Makefile.toml) for other targets

