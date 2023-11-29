# Introduction

> This section explains features which are not implemented yet.

The [builds.rs][] project aims to generate build artifacts for all crates
published on [crates.io][]. 

## Browse and download artifacts

The easiest way to find and download artifacts is through the web interface
at [builds.rs][].

## Download artifacts

You can also fetch artifacts using `curl`, for example in CI pipelines.

```
curl -sSL https://builds.rs/crates/mycrate/0.1.2/binaries/x86_64-unknown-linux-gnu > mycrate-0.1.2.tar.gz
```

[builds.rs]: https://builds.rs
[crates.io]: https://crates.io
