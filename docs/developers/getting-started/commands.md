# Commands

This section explains what tools and commands there are in this repository that
can help you launch and manage buildsrs.

## Justfile

This project uses the Just tool to help in running common commands. This tool allows
us to define shortcuts for common tasks in a `Justfile`. If you don't have `just`
installed, see the [Installing Just](./prerequisites.md#just) section.

The way that just works is that we can define commands in the `Justfile`. Commands
can have optional parameters. You can see which commands are available along with
some help text by running `just` without any arguments in the repository root.

This is a full list of all of the commands, and what they are used for.

| Name | Description |
| --- | --- |
| backend | Launches backend. |
| builder | Launches builder. |
| check | Run formatting an style checks. |
| ci | Run tasks similar to what the CI runs. |
| coverage | Generate test coverage report. |
| database-cli | Run database command-line interface. This is a tool that is defined in `database/` that allows you to manage the database. Commonly, you can use it to run migrations with `just database-cli migrate`. Run it without options to see what is available. |
| database-dump | Create a dump of the current database contents. This is useful for testing. |
| database-repl | Launch an interactive console to check the database. |
| docs | Build documentation. |
| format | Formats code. |
| frontend | Launch frontend. |
| registry-sync | Launches registry-sync service. |
| services | Launches services. |
| test | Runs unit tests. |

### Patterns

This explains some of the commands more in-depth, to give some context on what they
do and how they are meant to be used.

### Database Dump

While the migrations are tested in the unit tests, it can be difficult to ensure
that data which lies in the database can be properly migrated. For this reason,
there exists a command to create a dump of a locally running database which is
saved into the repository and can be used to create a unit test from.

```
# create database/dumps/latest.sql.xz
just database-dump
```

After taking such a dump, the database crate unit tests have a functionality to
create a unit test which restores this dump into a temporary database, runs all
migrations over it, and then check if the data is still accessible.

### Database REPL

When making changes to the database migrations or handlers, it may be possible
to break unit tests. Every unit test works by creating a temporary database, run
the migrations on it, execute the code in it and finally deleting the temporary
database. In case of an error, the temporary database is not deleted but kept in
order to be able to inspect it.

In that case, look for an output similar to this in the test results:

```
=> Creating database "test_jvqbcyqagfmuncq"
=> Run `just database-repl "test_jvqbcyqagfmuncq"` to inspect database
```

This output hints at the ability to use a command to inspect the database
after the test failure. Keep in mind that temporary databases are only kept
in case of an error in the test. 

Use the appropriate Just command to start a REPL that you can use to inspect
the database at the time which the error occured.

```
just database-repl test_jvqbcyqagfmunc
```

### Database CLI
