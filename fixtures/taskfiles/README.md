# Taskfile examples

These examples are mainly used as test fixtures for Stencila's internal `tasks` Rust crate. We use them to ensure that the `Taskfile` syntax can be parsed (and serialized) by Stencila as well as to confirm expected behavior from the `task` binary.

Most of the `Taskfile`s have a `default` task. You can run them using:

```sh
task --taskfile variables.yaml
```

or

```sh
stencila binary run task -- --taskfile variables.yaml
```

if you installed the `task` binary using Stencila.
