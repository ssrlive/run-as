# run-as

[![Build Status](https://github.com/ssrlive/run-as/workflows/Tests/badge.svg?branch=master)](https://github.com/ssrlive/run-as/actions?query=workflow%3ATests)
[![Crates.io](https://img.shields.io/crates/d/run-as.svg)](https://crates.io/crates/run-as)
[![License](https://img.shields.io/github/license/ssrlive/run-as)](https://github.com/ssrlive/run-as/blob/master/LICENSE)
[![rustc 1.56.0](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://img.shields.io/badge/rust-1.85%2B-orange.svg)
[![Documentation](https://docs.rs/run-as/badge.svg)](https://docs.rs/run-as)

> This crate is a fork of [runas](https://github.com/mitsuhiko/rust-runas)

A simple Rust library that can execute commands as root.

```rust
use run_as::Command;

let status = Command::new("rm")
    .arg("/usr/local/my-app")
    .status()
    .unwrap();
```

## License and Links

* [Documentation](https://docs.rs/run-as/)
* [Issue Tracker](https://github.com/ssrlive/run-as/issues)
* [Examples](https://github.com/ssrlive/run-as/tree/master/examples)
* License: [Apache-2.0](https://github.com/ssrlive/run-as/blob/main/LICENSE)
