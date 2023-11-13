# Builds.rs Documentation

[builds.rs][] is a service that builds artifacts for all crates published at
[crates.io][], the Rust community's crate registry.  [builds.rs][] takes all
crates published there and generates artifacts from them, such as executables
for different platforms. This makes it easy to use Rust tooling without needing
to compile it from source.

<center>
<img src="builder.svg" width="150" />
</center>

[builds.rs][] is written, run and maintained by a team of volunteers who are
passionate about the Rust language and the quality of tooling it has provided.
We want to do our little part in making this tooling available to as many
people as possible.

### Sections

This guide is split up into different sections for the different target
audiences that might read this guide. We recommend reading these sections
in the order that they are presented.

**[Users](summary.md)**

This section is for people that want to use [builds.rs][], for example to
download artifacts for Rust crates. It summarizes what [builds.rs][] does, how
it works, and how you can use it.

**[Crate Authors](authors)**

This section is for crate authors that would like to customize how
[builds.rs][] builds their crates. As a crate author, you can add metadata to
your crate's manifest that controls how your crate is built.

**[Developers](developers.md)**

This section is for anyone who would like to help maintain [builds.rs][]. It
explains how the project is structured, how you can run and test it locally,
and what you need to keep in mind when creating merge requests.

[builds.rs]: https://builds.rs
[crates.io]: https://crates.io
