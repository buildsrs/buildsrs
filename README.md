# builds.rs

[![pipeline status](https://gitlab.com/buildsrs/buildsrs/badges/main/pipeline.svg)](https://gitlab.com/buildsrs/buildsrs/-/pipelines)

Automated build system for all binary Rust crates on
[crates.io](https://crates.io).

The Rust community has built some incredible tools, but some of these are not
as accessible because the authors might not provide builds for them for all
relevant platforms, or because they require to be built from source.

The goal for this project is to make it easy to use Rust tools and applications
in the field by providing a convenient service to build binaries for multiple
platforms and architectures. 

This project itself is written in Rust and is open source to encourage
contribution.

## Status

This project is still under heavy development. We are currently aiming to get
the project into a deployed alpha state as soon as we can so that we can
collect some data and feedback.

## Getting Started

Documentation is still in the process of being written. 

Check out the [Development guide](docs/guides/development.md) for information
on how to get started developing this project.

The [Architecture overview](docs/guides/architecture.md) has important context
for how this project is currently organized in terms of services and code.

## License

[MIT](LICENSE.md).
