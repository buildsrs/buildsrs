# Registry Sync

This crate builds a single binary that is responsible for synchronizing the
crates index with the builds.rs database. It has two main tasks:

- Periodically fetch and crawl the crates.io index, adding new crates and
  versions and updating the yanked status on existing crate versions.
- For every added crate version, fetch it and check if it is a binary
  crate, as only binary crates need to be built by this backend. 
