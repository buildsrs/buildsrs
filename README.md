# builds.rs

[![pipeline status](https://gitlab.com/buildsrs/buildsrs/badges/main/pipeline.svg)](https://gitlab.com/buildsrs/buildsrs/-/pipelines)
[![documentation](https://img.shields.io/badge/docs-main-blue)](https://docs.builds.rs)
[![rustdoc](https://img.shields.io/badge/rustdoc-main-blue)](https://docs.builds.rs/rustdoc/buildsrs_backend/)
[![coverage](https://img.shields.io/badge/coverage-main-blue)](https://docs.builds.rs/coverage)

Project to provide binary builds automatically for all crates on [crates.io][].
Aims to provide binaries for multiple targets, driven by metadata specified in
the Cargo manifest.

The goal is to be able to easily fetch binary builds for any crate without needing
a Rust toolchain installed locally:

```
curl https://builds.rs/builds/ripgrep/latest/x86_64-unknown-linux-gnu
```

## Status

This project is currently under heavy development. An alpha version is expected
to be deployed at [builds.rs][] soon. There is a [Discord
server](https://discord.gg/kfSpdypU) for development-related communication.

## Getting Started

Read the [Developer Documentation](https://docs.builds.rs/developers/intro.html)
for information on how to get started with [builds.rs][].

## License

[MIT](LICENSE.md).

[crates.io]: https://crates.io
[builds.rs]: https://builds.rs
