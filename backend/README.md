# builds.rs backend

This crate is the backend part of [builds.rs][]. It offers an API that can be
used to search and view crate information, as well as download builds.

## Development

In order to launch the backend locally, you will need to run the following steps:

```
just database              # launch the database
just database-cli migrate  # run migrations
just backend               # launch backend
```

[builds.rs]: https://builds.rs
