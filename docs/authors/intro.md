# Introduction

This section of the [builds.rs][] documentation is aimed at crate developers.
[builds.rs][] is a service which will create build artefacts for the crates you
publish on [crates.io][] automatically and for free. The aim in doing so is to
make it as easy as possible to deploy the code you write, without depending on
you to create and maintain CI setups to build for different architectures.

You do not need to use [builds.rs][], in fact if your crates have a complex
build system then you may not want to use it at all. But if you do want to use
it, this section will tell you what you can do to make sure your crate builds
easily and cleanly and you can get the most out of the service.

## Usage

You may use [builds.rs][] in any way you like. You are free to link directly
to the builds. You do not need to attribute [builds.rs][] in any way. [builds.rs][]
will never charge money for the services it provides, nor will it ever interfere
with the way crates are built, such as by injecting code that is not a normal
part of the crate's build process.

[crates.io]: https://crates.io
[builds.rs]: https://builds.rs
