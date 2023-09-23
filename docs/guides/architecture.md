# Architecture

This guide explains the architecture of this project, both in terms of deployed
services as well as in terms of the code structure. This exists to give an overview
of how the system works as well as which component is responsible for what.

This project uses a microservice architecture, meaning that there are multiple
services that need to be deployed. This makes deploying and scaling easier. Care
is taken to avoid some of the pitfalls of a microservice architecture: services are
only split if it is absolutely necessary, to keep the amount of services that need
to be deployed to a reasonable amount. Currently, there are only two services that
need to be deployed (`backend` and `registry-sync`) for the system to work, and
and builders as needed.

Additionally, the API between services is not reimplemented in every service, but
defined once in common crates and reused. This, in addition to serde makes building
even complex communication protocols somewhat simple.

## Services

This project is deployed as a handful of containerized services. There are two
stateful components (a Postgres database and a Wasabi storage account), and the
rest of the components are stateless.

```mermaid
graph BT
    Storage[fa:fa-database Wasabi S3\nbuilds-production]
    Frontend[fa:fa-globe Frontend\nbuilds.rs]
    Frontend -->|HTTPS| Storage
    Frontend -->|HTTPS| Proxy
    subgraph Deployment
        direction BT
        Database[fa:fa-database Postgres Database]
        Backend[fa:fa-server builds.rs Backend]
        Backend  -->|SQL| Database
        Sync[fa:fa-download Registry Sync] -->|SQL| Database
        Proxy[fa:fa-globe Cloudflare CDN\napi.builds.rs] -->|HTTPS| Backend
        Builder[fa:fa-vial Builder] -->|WebSocket| Backend
    end
    Backend  -->|HTTP| Storage
```

### Backend

The backend is responsible for offering two APIs: the public REST API that the
frontend uses to fetch metadata, such as which crates and versions exist and
which artifacts have been built. The second API is for the builder instances
to connect and fetch build jobs, consisting of a WebSocket and a REST API for
uploading artifacts. This component tracks the number of downloads for each crate
and periodically writes this data to the database.

### Storage

Storage is handled by Wasabi. This is an external provider that offers an
S3-compatible API at reasonable prices and no egress fees.

### Database

Uses a [Postgres][postgres] database to store metadata. The database stores a
list of crates and crate versions that is synced to the list of crates on
<crates.io> using the Registry Sync service, a list of registered Builders, a
list of current or previous jobs and a list of artifacts for every crate
version.

### Frontend

The frontend is a Rust WebAssembly application written using the [Yew][yew]
framework, this is deployed as the main website for <builds.rs>. It talks to
the backend using a REST API.

### Registry Sync

The registry sync components keeps the system in sync with the list of crates
published on <crates.io>. To do this, it polls the [crates.io index][crates.io
index] and inserts any changes into the database directly.

### Builder

The builder is a component that fetches jobs from the backend, builds them
using [Docker][docker], and pushes the resulting binaries back into the
backend. This can be replicated as needed for parallel building.

## Crates

This project is split up into multiple sub-crates to allow for an easier
development experience. The goal is to compartmentalize responsibility and to
allow for sharing code between different services. By defining APIs in terms of
shared types that can be serialized and deserialized using [serde][serde], it
becomes quite easy to build stable and robust APIs between components.

```mermaid
graph BT
    Frontend[buildsrs_frontend<br/><i>Web user interface</i>]
    Backend[buildsrs_backend<br/><i>API for frontend and builder</i>]
    Common[buildsrs_common<br/><i>Common type definitions</i>]
    Database[buildsrs_database<br/><i>Database interactions</i>]
    Protocol[buildsrs_protocol<br/><i>Builder protocol types</i>]
    Builder[buildsrs_builder<br/><i>Builds crate artifacts</i>]
    DatabaseCli[buildsrs_database CLI<br/><i>Database management tool</i>]
    RegistrySync[buildsrs_registry_sync<br/><i>Registry sync service</i>]

    Database-->Common
    Backend-->Database
    Backend-->Common
    Backend-->Protocol
    Builder-->Protocol
    Frontend-->Common
    DatabaseCli-->Database
    RegistrySync-->Database
```

### `buildsrs_database`

This crate defines all interaction with the database. Services may not directly
interact with the database by any other means (sending SQL statements). This
crate includes tooling for building comprehensive unit tests for database
interactions and can detect when connecting to a database in an invalid state
(such as not properly migrated) on service launch.

This crate also includes a CLI utility that can be used for administrative
purposes, such as manually running migrations, registering Builders, setting
builder permissions and manually disabling crates or builders.

### `buildsrs_backend`

This crate implements the backend, it connects to the database using the
`buildsrs_database` crate and uses the Axum crate for offering the REST and
WebSocket APIs.

### `buildsrs_common`

This crate contains shared types, such as API request and response types that
are used by multiple crates.

### `buildsrs_frontend`

The frontend crate defines a Rust WebAssembly application that is designed to
run in the browser and be the main frontend for <builds.rs>. This uses the
types defined in the `common` crate to communicate with the backend.

### `buildsrs_protocol`

Communication with the builder and the backend is a bit complex, because of the
need for authentication and for the need to sign artifacts electronically to
form a chain of trust that can be externally validated.

For this reason, the `protocol` crate defines the protocol in terms of types
(enums of messages that may be sent). These are unit tested.

### `buildsrs_builder`

This crate defines the builder component, which can be dynamically deployed as
needed and connects to the backend to fetch jobs. It uses the
`buildsrs_protocol` crate for the types needed to send messages to the backend.

### `buildsrs_registry_sync`

The `buildsrs_registry_sync` crate defines the service which syncronizes the
list of published crates on the [crates.io index][] with the database.

[postgres]: https://www.postgresql.org/
[crates.io index]: https://github.com/rust-lang/crates.io-index
[crates.io]: https://crates.io/
[docker]: https://docs.docker.com/engine/install/
[rustup]: https://rustup.rs/
[just]: https://github.com/casey/just
[trunk]: https://trunkrs.dev/
[wasabi]: https://wasabi.com/
