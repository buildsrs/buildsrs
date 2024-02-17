# Components

This chapter explores the architecture of this project both in terms of
deployed services as well as in terms of crates.

## Services

```mermaid
graph BT
    Storage[fa:fa-database Storage]
    Database[fa:fa-database Database]
    Frontend[fa:fa-globe Frontend]
    subgraph builds.rs
        Backend[fa:fa-server Backend]
        Sync[fa:fa-download Registry Sync] 
        Builder[fa:fa-wrench Builder]
        Builder --> Backend
    end
    Sync --> Database
    Backend  --> Database
    Backend  --> Storage
    Frontend --> Storage
    Frontend --> Backend
```

This project uses somewhat of a [microservice][] architecture, although one
could argue that since most of the action happens in the single backend
component, it is more of a monolith. 

Every component that needs deployment is built into a Docker container in the
CI, and then deployed on a cluster. 

There are only two components that are external and persistent: storage and the
database. These are abstracted away in the code. The storage component is
usually any S3-compatible storage provider, and the database is typically a
Postgres database.

## Crates

```mermaid
graph BT
    frontend[buildsrs_frontend<br/><i>Web user interface</i>]
    backend[buildsrs_backend<br/><i>API for frontend and builder</i>]
    common[buildsrs_common<br/><i>Common type definitions</i>]
    database[buildsrs_database<br/><i>Database interactions</i>]
    protocol[buildsrs_protocol<br/><i>Builder protocol types</i>]
    builder[buildsrs_builder<br/><i>Builds crate artifacts</i>]
    registry_sync[buildsrs_registry_sync<br/><i>Registry sync service</i>]
    storage[buildsrs_storage<br/><i>Storage</i>]

    database-->common
    backend-->database
    backend-->common
    backend-->storage
    backend-->protocol
    builder-->protocol
    frontend-->common
    registry_sync-->database

    click database "/rustdoc/buildsrs_database"
    click backend "/rustdoc/buildsrs_backend"
    click builder "/rustdoc/buildsrs_builder"
    click registry_sync "/rustdoc/buildsrs_registry_sync"
    click protocol "/rustdoc/buildsrs_protocol"
    click frontend "/rustdoc/buildsrs_frontend"
    click common "/rustdoc/buildsrs_common"
    click storage "/rustdoc/buildsrs_storage"
```

Code-wise, this project is a [Cargo workspace][workspace] with multiple crates.
Every target that needs to be built is it's own crate. In addition to that, any
code that needs to be used from multiple target crates is split out into it's
own crate.

The next chapters will deal with each of these components, explaining what they
do and how they are related to the other components.

[workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[microservice]: https://martinfowler.com/articles/microservices.html
