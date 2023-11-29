# Overrides

> This section explains features which are not implemented yet.

In order to get crates to build which you have already published, we have
the ability to override incorrect metadata for existing crates. For this,
you can write the configuration in much the same way as you could in your
Cargo manifest, and it will be overlaid to the metadata that exists in your
crate.

These overlay configurations are managed in a Git repository for
collaboration and transparency. 
