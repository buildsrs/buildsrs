# Maintenance

This checklist contains tasks that should be regularly performed, and a
suggested interval that they should be performed at. None of these tasks are
critical, but it makes sense to keep ahead of things.

## Weekly

### Update dependencies

Dependency versions are specified as bounds in the Cargo manifests, but
resolved in the [Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html) file. Occasionally, the resolved dependency versions should be updated
to use the latest versions.

To do so, use Cargo to update the lock file, make sure nothing breaks by
running tests afterwards. 

```
cargo update
just ci
```

If everything works (no errors), create a merge request with the changes.

### Upgrade dependencies

Occasionally, dependencies will publish new versions which are not
backwards-compatible. These upgrades tend to involve a bit more work, because
the code often needs to be adjusted.

You can use the tool [`cargo-outdated`](https://github.com/kbknapp/cargo-outdated)
to check which dependencies are outdated:

```
cargo outdated
```

For each of the outdated dependencies, you can try to manually upgrade them
by updating their version in the `Cargo.toml` and modifying the code. Check
that everything works locally with:

```
just ci
```

If everything works (no errors), create a merge request with the changes.

## Monthly

### Update Rust toolchain

The team behind Rust regularly releases a new version of the Rust toolchain.
For stability reasons, we currently hardcode which version we build and test
against in the CI. 

When a new version is released, update in the repository:

- Adjust `RUST_VERSION` in `.gitlab-ci.yml` to the new version
- Adjust the Rust version in each of the `Dockerfile` (in `backend`, `builder`, `registry-sync`, `database`) to the new version

Run tests to make sure nothing broke:

```
just ci
```

If everything works (no errors), create a merge request with the changes.

### Update CI tooling

In the CI, we use bunch of tooling:

- [sccache](https://github.com/mozilla/sccache)
- [trunk](https://github.com/thedodd/trunk)
- [mdbook](https://github.com/rust-lang/mdBook)
- [mdbook-mermaid](https://github.com/badboy/mdbook-mermaid)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [cargo-hack](https://github.com/taiki-e/cargo-hack)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)

For each of these tools, we have a variable such as `SCCACHE_VERSION` in the
`.gitlab-ci.yml` which tells the CI which version of the tool to download and
use. Occasionally, these tools get new releases, in which case we should update
to the most recent version of the tool.

For every tool:

- Check if there is a new version. If not, skip this tool.
- Update the version variable in `.gitlab-ci.yml` to point to the new version
  of the tool
- Check if it passes CI

Create a merge request with all the upgrades that were successful. Feel free to
indicate which dependencies you were not able to upgrade, and why.

## Yearly

### Review new Clippy lints

Every so often, the Clippy team releases new
[lints](https://rust-lang.github.io/rust-clippy/master/). It makes sense to
check them out occasionally and test if some of the newly added ones make sense
to add to the lint configuration in the `Cargo.toml`.

When adding new lints, run the checks to make sure existing code passes them,
if not you may have to fix the code.

```
just check
```

Once you had added some lints that appear to make sense and have adjusted the
code, feel free to create a merge request with the changes.

### Review test coverage minimum

In the CI, it is possible to set a minimum test coverage percentage. This is
a value that should 

- Find out current test coverage at the [coverage report](/coverage).
- Adjust the `fail-under-lines` setting in the `.gitlab-ci.yml` to be closer to
  the current test coverage to prevent it regressing.

Create a merge request and make sure that the pipeline succeeds.

