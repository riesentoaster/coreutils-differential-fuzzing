# Differential Fuzzing on coreutils

Clone the repository using `git clone --recurse-submodules https://github.com/riesentoaster/coreutils-differential-fuzzing.git`

## Fuzzer

This project is written in rust/cargo (make sure you have a [current version installed](https://doc.rust-lang.org/cargo/getting-started/installation.html)) and uses [cargo make](https://sagiegurari.github.io/cargo-make/) to document and automate the build steps.

To build all the necessary artifacts
- Navigate to the fuzzer directory: `cd fuzzer`
- Run `cargo make fuzzer` (this may take a few minutes, since GNU's and coreutils' version of coreutils are built)

Other targets include:
- `cargo make fuzzer_gnu` to only run on GNU's version of coreutils (without differential fuzzing)
- `cargo make fuzzer_uutils` to only run uutils' version
- `cargo make run` to directly run the fuzzer (resp. `cargo make run_gnu`/`cargo make run_uutils` to directly run the fuzzer on one implementation only)
- Check [`Makefile.toml`](./fuzzer/Makefile.toml) for other targets

## Report

Read the report [here](./report/out/index.pdf). Its artifacts are in the `report` subdirectory.