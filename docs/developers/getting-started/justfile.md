# Justfile

## Formatting

In order to format all of the code in this repository, you can use the `format`
Just target:

```
just format
```

This requires you to have installed Rust nightly as it uses the nightly version
of the formatter, which accepts more options.

## Database

The database is something which has a state and that state needs to be carefully
managed. For this reason, it takes special care to ensure correctness. There are
specific commands useful for helping test and inspect the database.

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

