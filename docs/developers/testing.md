# Testing

Testing is an integral part of the development process of this project. The aim
is to make sure that all context is carefully encoded in the form of tests, to
make sure that this project can grow without being dependent on communicating
constraints and thoughts between developers.

To ensure that testing is being done thoroughly, some thought has been put into
measuring it and designing this project in a way that facilitates testing.

## Coverage

In order to measure the progress of the testing effort of this project, test
coverage is measured for every commit in the CI pipeline. The coverage report
is available [here](/coverage/).

The goal is for this coverage to be almost perfect. In the CI, we enforce a
minimum coverage percentage that gets adjusted as coverage grows to prevent
regressions.

## State

Another tricky issue when building tests is dealing with state. What we did for
this project is to build the stateful parts, which are the database and the
storage layer, in a way that is generic so that the implementations can be
swapped out. At the same time, they are built in a way that it is possible to
create a new, ephemeral instance for every unit test.

Both stateful aspects can be run as Docker containers, and the requisite files
are present in this repository, so that running tests is as simple as:

```
just services up
just test
```

Building the project in this way should reduce the friction in running tests
locally and in making sure new features come with tests.
