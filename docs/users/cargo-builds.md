# Cargo Builds

> This section explains features which are not implemented yet.

This project also has a CLI that you can install which integrates with the
`cargo` build tool. You can use this to fetch binaries.

## Installation

The easiest way to install this tool is using `cargo` itself.

```
cargo install cargo-builds
```

Once you have it installed, you should be able to call it like this:

```
cargo builds
```

## Usage

### Fetch crate

By default, the fetch command will fetch binary artifacts for the latest
version and current architecture. However, you can use command-line arguments to override
those defaults.

```
cargo builds fetch serde
```

### Test crate build

You can also use this tool to test building of your local package.

```
cargo builds build
```
