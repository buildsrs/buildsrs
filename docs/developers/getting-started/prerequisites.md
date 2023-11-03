# Prerequisites

Developing in this project requires some tooling to be installed locally. We
try to require as few locally installed tools as possible, however these four
have proven to be worth the effort to install them.

- [Rustup][rustup]: Manages your local Rust installation.
- [Just][just]: Runner for custom commands.
- [Trunk][trunk]: Helps you to build Rust WebAssembly frontend.
- [Docker][docker]: Container runtime used to launch local services.

Optionally, you can also install these two tools. They are not required for
development, but they enable you to build certain things that you otherwise
cannot.

- [cargo-llvm-cov][cargo-llvm-cov]: Optional, used to determine test coverage.
- [mdbook][mdbook]: Optional, used to build documentation.

Here are explanations for what each tool does and a quick guide to getting it
installed.

## Rustup

[Rustup][rustup] manages Rust installations. It is able to keep the Rust
toolchain updated.  While it is not strictly required, it is the recommended
way to install Rust on your system as it lets us easily lock this project to a
specific version of Rust.

On a Unix-like system, you can install it like this. Please follow the
instructions that it shows, for example you may have to open a new shell
session to be able to use it.  For other systems, check the [website][rustup]
for more information.

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once you have installed Rustup, you should be able to build the code in the
repository by running this command.  If this succeeds, then you have
successfully installed Rustup.

```
cargo build
```

If you intend to build the frontend as well, you likely want to add the
WebAssembly target for Rust.  You can do it by running this command:

```
rustup target add wasm32-unknown-unknown
```

The easiest way to test if this works is by heading to the [Trunk](#trunk)
section, and installing and testing it by building the frontend.

## Trunk

[Trunk][trunk] is a tool that helps with building Rust WebAssembly frontends.
It wraps around `cargo` for building the WebAssembly and bundles the resulting
raw binaries into a working website, ready to be consumed by the browser.

We use it to build the frontend for builds.rs, which is written in Rust using
the [Yew][yew] framework. If you do not want to run the frontend, you do not
need to install Trunk on your system.

If you already have a Rust toolchain installed, one easy way to get Trunk is by
installing it using cargo.

```
cargo install trunk
```

In order to use Trunk, you also need to add the WebAssembly target for Rustup.
The [Rustup](#rustup) section will tell you how to do this.

You can verify that your installation works by running this command:

```
cd frontend && trunk build
```

Make sure you update it occasionally by re-running the installation command as
it is still being developed and gaining new features.

## Docker

[Docker][docker] is a containerization platform that allows you to package,
distribute, and run applications and their dependencies in isolated, portable
environments.  It is used to run services (such as the database) in a way that
does not require you to install it locally, but rather in a prepackaged
container.

The [installation](https://docs.docker.com/engine/install) process depends on the
platform that you are using, but the simplest installation method if you are
on Debian is by using APT:

```
apt install docker.io apparmor-utils
adduser $USER docker
```

Make sure that you also install Docker Compose, as that is needed to launch local
services.

## Just

[Just][just] is a command runner, similar to how Makefiles are often used. It
offers less complexity compared to Makefiles and has some neat features,
including command arguments and built-in documentation.

If you already have a Rust toolchain installed, one easy way to get Just is by
installing it using cargo.

```
cargo install just
```

There is one `Justfile` in this repository, and if you run only `just` you will
see a list of targets that are defined.

```
just --list
Available recipes:
    backend                # launch registry sync
    builder                # launch builder
    coverage               # generate test coverage report
    database               # start postgres database
    database-cli *COMMAND  # run database cli
    database-dump NAME='latest' DATABASE='postgres' # save database dump
    database-repl DATABASE # start repl with specified postgres database
    database-test          # test database
    format                 # Format source with rustfmt nightly
    frontend               # launch frontend
    list                   # list targets and help
    registry-sync          # launch registry sync
    test filter=''         # run all unit tests
```

Most of these are shortcuts to launch specific components (`database`,
`backend`, `builder`, `registry-sync`), or do specific actions (`test`,
`coverage`, `format`). These commands are further explained in the rest of
this guide.

## Cargo llvm-cov

[Cargo llvm-cov][cargo-llvm-cov] is a tool that lets us build test coverage reports
to measure how good of a job we are doing in testing the code base. It is not required
for development, but can be a handy tool.

You can install it with cargo like this:

```
cargo install llvm-cov
```

To test it, you can use the `coverage` target by running:

```
just coverage
```

If this runs without producing errors, then you know that the tool is properly installed.

## mdBook

[mdBook][mdbook] is a tool used to build the documentation for build.rs. It
takes as input the markdown files found in the `docs/` folder of the
repository, and produces this nice documentation page.

If you want to work on improving the documentation, it is recommended that you
install this locally so you can render the documentation.

You can install it using `cargo` by running this command:

```
cargo install mdbook
```

You can verify that it does work by building the documentation locally, like
this:

```
mdbook build
```

If this command runs, then you know that it is working.

## Troubleshooting

This section is dedicated to any common issues one might encounter with these
tools.  If you run into any issues, feel free to open an [issue][issues] in our
issue tracker and let us know about it. We generally cannot help you too much
with troubleshooting your local environment, but we are happy to fix incorrect
documentation or document common issues here.

[issues]: https://gitlab.com/buildsrs/buildsrs/-/issues
[docker]: https://docs.docker.com/engine/install/
[rustup]: https://rustup.rs/
[just]: https://github.com/casey/just
[trunk]: https://trunkrs.dev/
[yew]: https://yew.rs/
[cargo-llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov
[mdbook]: https://github.com/rust-lang/mdBook
