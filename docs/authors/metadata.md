# Metadata

> This section explains features which are not implemented yet.

[builds.rs][] aims to do the right thing by default, and will try it's best
to figure out how to build your crate. However, it is not perfect. In some
situations, it needs extra information to tell it how your crate needs to be
built. 

For those situations, it is possible to add metadata to your Cargo manifest
which [builds.rs][] can parse and use in the build process. This chapter describes
what that metadata looks like and how you can use it.

In general, any and all metadata you can set will be under the
`package.metadata.buildsrs` table.

## Features

Using the `features` array, you can set the features that are enabled when building
your crate. If you do not specify this, then the crate's default feature set will
be used.

```toml
[package.metadata.buildsrs]
features = ["feature-x", "feature-y"]
binaries = ["main", "other"]
targets = ["x86_64-unknown-linux-musl"]
```

[builds.rs]: https://builds.rs
