# Running tests

Testing is one of the most important parts of the process of developing this
software. Tests serve both as documentation to some extent and they allow for
teams to implement features without needing to communicate all hidden
assumptions, they can instead be encoded in the form of unit tests.

The approach that this project is taking is by writing as many unit tests as
are necessary, and using coverage reporting to measure how the test coverage
changes over time. All new features should come with matching tests, if
possible.

## Services

In order to be able to run the tests, you must first launch the required services.
You can launch them using the `services` command:

```
# launch services
just services
```

If you want to tear them down and delete any state, you can use this command
with the `down` subcommand, like this:

```
# delete services
just services down
```

### Testing

There are two targets that are useful for running tests. Both of these targets
require a running database, but they do not require the database to be migrated
as they create temporary virtual databases.

```
# run all unit tests
just test

# run only database unit tests
just database-test
```

### Coverage

For estimating test coverage, `llvm-cov` is used which needs to be separately
installed. This uses instrumentation to figure out which parts of the codebase
are executed by tests and which are not.

There is a useful target for running the coverage tests.

```
just coverage
```

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

