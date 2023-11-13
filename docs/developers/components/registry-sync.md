# Registry Sync

The registry sync components keeps the system in sync with the list of crates
published on [crates.io][]. To do this, it polls the [crates.io
index][crates.io index] and inserts any changes into the database directly.

## Services

```mermaid
graph BT
    database[Database]
    registry-sync[Registry Sync]
    registry-sync --> database

    click database "./database.html"
```

The Registry Sync service connects directly to the database to keep it in sync.
It has no other dependencies.

## Crates

```mermaid
graph BT
    database[buildsrs_database]
    registry-sync[buildsrs_registry_sync]

    registry-sync --> database

    click database "/rustdoc/buildsrs_database"
    click registry-sync "/rustdoc/buildsrs_registry_sync"
```

It is implemented in the [buildsrs_registry_sync][] crate. It depends on the
[buildsrs_database][] crate for database interactions.

[crates.io index]: https://github.com/rust-lang/crates.io-index
[crates.io]: https://crates.io/
[buildsrs_database]: /rustdoc/buildsrs_database
[buildsrs_registry_sync]: /rustdoc/buildsrs_registry_sync
