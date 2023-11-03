# Components

To get started, you need to run four components: the database, the backend, the
registry-sync service and the builder.

Ideally, you can start every component in a different `tmux` pane, or terminal
tab, so you can observe what they are doing.

## Database

The database, which is a Postgres database, can be launched with the following
two commands. The first command starts the database process, and the second one
runs migrations on it.

```
# launch database
just database

# run migrations (run this in a different tab)
just database-cli migrate
```

That latter command uses the database CLI, which offers subcommands that are
useful for interacting with the database, such as registering runners.

If you make changes to the database migrations, you may have to reset the
database in order to be able to apply them. To do this, simply cancel the
launched database and re-launch it, as it is not persistent.

## Backend

The backend hosts the API for the frontend and for the runners to connect. By default,
it will listen locally on `localhost:8000` for API requests. It requires the database
to be running and migrated.

```
# launch backend
just backend
```

## Registry Sync

In order to synchronize the database with the crates on <crates.io>, you need to
launch the registry sync service. This requires a running and migrated database.

```
# launch registry sync
just registry-sync
```

## Builder

The builder is the component that actually builds crates. You need to launch
the backend before you can launch the builder. You will also need to register
it with the database. Here is how to do that:

```
# register builder with backend
just database-repl builder add ~/.ssh/id_ed25519.pub
# launch builder
just builder
```

