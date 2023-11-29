# Testing

> This section explains features which are not implemented yet.

Testing the metadata for [builds.rs][] is quite important, you likely do not
want to publish crates with broken metadata. For this reason, the `cargo-builds`
tool ships with the ability to locally build your crate's artifacts exactly
the same way that [builds.rs][] would. 

To use this, run this command:

```
cargo builds build
```

What this command will do is parse your Cargo manifest and build all crate
artifacts just like [builds.rs][] would build them. They will be placed inside
of `target/buildsrs/`. Note that this will call `cargo package` to crate a
package containing everything that would exist if you were to publish your
crate, and it needs access to Docker for running the build steps.

[builds.rs]: https://builds.rs
