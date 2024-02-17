# Running locally

It should be relatively straightforward to run buildsrs locally. To do so, you
need to run a few components:

- services
    - database (postgres, stores metadata)
    - minio (S3-compatible API for storing builds)
- backend (serves API)
- registry-sync (synchronizes crates from crates.io with database)
- builder (fetches jobs and builds crates)

The only thing you need to get these running is having Docker running on your
system.  Docker is not necessary, but it simplifies running the services that
the stack needs to talk to.

## Services

To launch the services that buildsrs needs to run locally, the easiest approach
is to run them using Docker. There is a `docker-compose.yml` file in the
repository and a Just target. You should be able to launch them like this:

```
just services
```

In order to use the database, you will need to run migrations. There is a CLI
tool in the `buildsrs-database` crate that you can use for this. You can run
them like this:

```
just database-cli migrate
```

Once you have launched the services and run the migration, your setup is ready.

If you make changes to the database migrations, you may have to reset the
database in order to be able to apply them. To do this, simply cancel the
launched database and re-launch it, as it is not persistent.

## Backend

The backend hosts the API for the frontend and for the runners to connect. By
default, it will listen locally on `localhost:8000` for API requests. It
requires the database to be running and migrated for it to run.

```
just backend
```

## Registry Sync

In order to synchronize the database with the crates on [crates.io][], you need
to launch the registry sync service. This requires a running and migrated
database.

```
just registry-sync
```

## Builder

The builder is the component that actually builds crates. You need to launch
the backend before you can launch the builder. You will also need to register
it with the database. Here is how to do that:

```
just database-cli builder add ~/.ssh/id_ed25519.pub
just builder
```

The builder uses SSH keys to authenticate with the backend. You can use any SSH
key, however by default it can use your local `ed25519` key. If you do not have
a local `ed25519` key, you can create one by running this and pressing enter
on any question the tool asks:

```
ssh-keygen -t ed25519
```

[crates.io]: https://crates.io
