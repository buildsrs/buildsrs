# Development

This guide explains how to get started for developing on this project.

## Prerequisites

Developing in this project requires some tooling to be installed locally.  We
try to require as few locally installed tools as possible, however these four
have proven to be worth the effort to install them.

- [Rustup][rustup]
- [Just][just]
- [Trunk][trunk]
- [Docker][docker]

Here are explanations for what each tool does and a quick guide to getting it
installed.

### Rustup

[Rustup][rustup] manages Rust installations. It is able to keep the Rust
toolchain updated.  While it is not strictly required, it is the recommended
way to install Rust on your system as it lets us easily lock this project to a
specific version of Rust.

On a Unix-like system, you can install it like this. For other systems, check
the [website][rustup] for more information.

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Just

[Just][just] is a command running, similar to how Makefiles are often used. It offers
less complexity compared to Makefiles and has some neat features, including
command arguments and built-in documentation.

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

### Trunk

[Trunk][trunk] is a tool that helps with building Rust WebAssembly frontends.
It wraps around `cargo` for building the WebAssembly and bundles the resulting
raw binaries into a working website, ready to be consumed by the browser.

We use it to build the frontend for <builds.rs>, which is written in Rust using
the [Yew][yew] framework. If you do not want to run the frontend, you do not
need to install Trunk on your system.

If you already have a Rust toolchain installed, one easy way to get Trunk is by
installing it using cargo.

```
cargo install trunk
```

If you encounter errors using Trunk, consider updating it as it is still being
developed and gaining new features.

### Docker

Docker is a containerization platform that allows you to package, distribute,
and run applications and their dependencies in isolated, portable environments.
It is used to run auxillary tools (such as the database) in a way that does not
require you to install it locally, but rather in a prepackaged container.

The [installation](https://docs.docker.com/engine/install) process depends on the
platform that you are using, but the simplest installation method if you are
on Debian is by using APT:

```
apt install docker.io apparmor-utils
adduser $USER docker
```

### cargo-llvm-cov

cargo-llvm-cov is an optional dependency that is used for estimating unit test
coverage. It can be installed like this:

```
cargo install llvm-cov
```

## Components

To get started, you need to run four components: the database, the backend, the
registry-sync service and the builder.

Ideally, you can start every component in a different `tmux` pane, or terminal
tab, so you can observe what they are doing.

### Database

The database, which is a Postgres database, can be launched with the following
two commands. The first command starts the database process, and the second one
runs migrations on it.

```
# launch database
just database

# run migrations (run this in a different tab)
just database-cli migrate
```

That latter command uses the database CLI, which offers subcommands that are
useful for interacting with the database, such as registering runners.

If you make changes to the database migrations, you may have to reset the
database in order to be able to apply them. To do this, simply cancel the
launched database and re-launch it, as it is not persistent.

### Backend

The backend hosts the API for the frontend and for the runners to connect. By default,
it will listen locally on `localhost:8000` for API requests. It requires the database
to be running and migrated.

```
# launch backend
just backend
```

### Registry Sync

In order to synchronize the database with the crates on <crates.io>, you need to
launch the registry sync service. This requires a running and migrated database.

```
# launch registry sync
just registry-sync
```

### Builder

The builder is the component that actually builds crates. You need to launch
the backend before you can launch the builder. You will also need to register
it with the database. Here is how to do that:

```
# register builder with backend
just database-repl builder add ~/.ssh/id_ed25519.pub
# launch builder
just builder
```

## Testing

Testing is one of the most important parts of the process of developing this
software. Tests serve both as documentation to some extent and they allow for
teams to implement features without needing to communicate all hidden
assumptions, they can instead be encoded in the form of unit tests.

The approach that this project is taking is by writing as many unit tests as
are necessary, and using coverage reporting to measure how the test coverage
changes over time. All new features should come with matching tests, if
possible.

### Testing

There are two targets that are useful for running tests. Both of these targets
require a running database, but they do not require the database to be migrated
as they create temporary virtual databases.

```
# run all unit tests
just test

# run only database unit tests
just database-test
```

### Coverage

For estimating test coverage, `llvm-cov` is used which needs to be separately
installed. This uses instrumentation to figure out which parts of the codebase
are executed by tests and which are not.

There is a useful target for running the coverage tests.

```
just coverage
```

## Database

The database is something which has a state and that state needs to be carefully
managed. For this reason, it takes special care to ensure correctness. There are
specific commands useful for helping test and inspect the database.

### Database Dump

While the migrations are tested in the unit tests, it can be difficult to ensure
that data which lies in the database can be properly migrated. For this reason,
there exists a command to create a dump of a locally running database which is
saved into the repository and can be used to create a unit test from.

```
# create database/dumps/latest.sql.xz
just database-dump
```

After taking such a dump, the database crate unit tests have a functionality to
create a unit test which restores this dump into a temporary database, runs all
migrations over it, and then check if the data is still accessible.

### Database REPL

When making changes to the database migrations or handlers, it may be possible
to break unit tests. Every unit test works by creating a temporary database, run
the migrations on it, execute the code in it and finally deleting the temporary
database. In case of an error, the temporary database is not deleted but kept in
order to be able to inspect it.

In that case, look for an output similar to this in the test results:

```
=> Creating database "test_jvqbcyqagfmuncq"
=> Run `just database-repl "test_jvqbcyqagfmuncq"` to inspect database
```

This output hints at the ability to use a command to inspect the database
after the test failure. Keep in mind that temporary databases are only kept
in case of an error in the test. 

Use the appropriate Just command to start a REPL that you can use to inspect
the database at the time which the error occured.

```
just database-repl test_jvqbcyqagfmunc
```

## Maintenance

This section explains how to do some maintenance tasks in the repository.

### Formatting

In order to format all of the code in this repository, you can use the `format`
Just target:

```
just format
```

This requires you to have installed Rust nightly as it uses the nightly version
of the formatter, which accepts more options.

[docker]: https://docs.docker.com/engine/install/
[rustup]: https://rustup.rs/
[just]: https://github.com/casey/just
[trunk]: https://trunkrs.dev/
[yew]: https://yew.rs/
