# Taskfile examples

These examples are mainly used as test fixtures for Stencila's internal `tasks` Rust crate. We use them to ensure that the `Taskfile` format can be parsed (and serialized) by Stencila and to test extensions to the `Taskfile` format, such as `schedule` and `watches` operate as expected.

Most of the `Taskfile`s have a `default` task. You can run them using:

```sh
stencila tasks run default --taskfile variables.yaml
```

or, using `task` directly:

```sh
task --taskfile variables.yaml
```
