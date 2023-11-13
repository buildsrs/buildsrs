# Components

This chapter explores the architecture of this project both in terms of deployed services
as well as in terms of crates.

## Services

```mermaid
graph BT
    Storage[fa:fa-database Storage]
    subgraph Deployment
        direction BT
        Database[fa:fa-database Database]
        Sync[fa:fa-download Registry Sync] --> Database
        Backend[fa:fa-server Backend]
        Backend  --> Database
        Builder[fa:fa-wrench Builder] --> Backend
    end
    Backend  --> Storage
    Frontend[fa:fa-globe Frontend]
    Frontend --> Storage
    Frontend --> Backend
```

This project uses a microservices architecture. Every component that needs deployment
is built into a Docker container in the CI, and then deployed on a cluster. There are
only two components that are persistent: storage and the database. The storage component
is usually any S3-compatible storage provider, and the database is typically a Postgres
database.

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

Code-wise, this project is a Cargo workspace with multiple crates. Every target
that needs to be built is it's own crate. In addition to that, any code that
needs to be used from multiple target crates is split out into it's own crate.

The next chapters will deal with each of these components, explaining what they do
and how they are related to the other components.

