# Stencila Tasks

## About

Welcome to the reference documentation for Stencila Tasks. This document provides a brief overview and links to documentation for each task in the Stencila Task library. See the [tutorial](../../tutorial/tasks) and [howto](../../howto/tasks) sections for tasks for more.

## Introduction

Stencila Tasks provide a way for you do define what gets done, and when, in your project. They are built on top of the [`Taskfile`](https://taskfile.dev/usage) format and [`task`](https://github.com/go-task/task), a `Taskfile` runner implemented in Go. Stencila Tasks is a library of `Taskfiles` containing tasks commonly used for running and publishing executable documents. Stencila Tasks also include extensions to the `Taskfile` format to support auto-generation of tasks, running them (or not) based on file changes, and running them according to a time schedule.

The Stencila CLI has a [`tasks` command](../cli/tasks) which provides subcommands for working with `Taskfiles` including:

- `list`: list, filter, and search for, tasks within the Stencila Tasks library or a project `Taskfile`
- `detect`: automatically detect which tasks are required for a project and add them to a project `Taskfile`
- `run`: manually run one or more tasks within a project `Taskfile`
- `watch`: automatically run tasks within a project `Taskfile` when there are changes in the project

The Stencila Rust library has a [`tasks` crate](https://github.com/stencila/stencila/tree/master/rust/tasks) provides Rust `struct`s and functions for working with the `Taskfile` format and interfacing with the `task` Go binary.

## Components

This section describes the components of a [`Taskfile`](#taskfile):

- [`Task`](#task): for defining a task
- [`Command`](#command): for defining a command to be run as part of a `Task`
- [`Precondition`](#precondition): for defining a condition that must be met for a `Task` to run
- [`Dependency`](#dependency): for declaring a dependency between `Task`s
- [`Variable`](#variable): for declaring `Taskfile` and environment variables
- [`Include`](#include): for including other `Taskfile`s

The order of attributes in each of the following tables is the recommended order to use when writing a `Taskfile`. Note also, that for several components, YAML syntax shortcuts are available.

These tables include Stencila's extension attributes which are, at present, not supported by `task`.
Consult the `Taskfile` [schema docs](https://taskfile.dev/api/#schema) for a canonical reference. Some wording differs.

### Taskfile

| Attribute  | Type                                    | Default       | Description                                                                                                    |
| ---------- | --------------------------------------- | ------------- | -------------------------------------------------------------------------------------------------------------- |
| `version`  | `String`                                |               | The version of the `Taskfile` schema.                                                                          |
| `includes` | `Map<String,`[`Include`](#include)`>`   |               | Additional `Taskfile`s to be included.                                                                         |
| `output`   | `String`                                | `interleaved` | Mode for controlling task output. Options: `interleaved`, `group` and `prefixed`.                              |
| `method`   | `String`                                | `checksum`    | Default dependency resolution method. Can be overridden by tasks. Options: `checksum`, `timestamp` and `none`. |
| `silent`   | `Boolean`                               | `false`       | Default `silent` attribute for tasks. If `false`, can be overridden by tasks.                                  |
| `run`      | `String`                                | `always`      | Default "run" option for this Taskfile. Options: `always`, `once` and `when_changed`.                          |
| `vars`     | `Map<String,`[`Variable`](#variable)`>` |               | Global template variables.                                                                                     |
| `env`      | `Map<String,`[`Variable`](#variable)`>` |               | Global environment variables.                                                                                  |
| `dotenv`   | `Array<String>`                         |               | Dotenv files to be included.                                                                                   |
| `tasks`    | `Map<String,`[`Task`](#task)`>`         |               | Task definitions.                                                                                              |

### Include

| Attribute  | Type      | Default                                | Description                                                                                                                                         |
| ---------- | --------- | -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `taskfile` | `String`  |                                        | Path of `Taskfile` or directory to be included. If a directory, the file named `Taskfile.yml` or `Taskfile.yaml` inside that directory is included. |
| `dir`      | `String`  | The directory of the parent `Taskfile` | Working directory of the included tasks when they are run.                                                                                          |
| `optional` | `Boolean` | `false`                                | If `true`, no errors will be thrown if the specified file does not exist.                                                                           |

You can specify an `Include` using a single string for the `taskfile` attribute. For example,

```yaml
includes:
  another: ./path/to/another.yaml
```

is equivalent to,

```yaml
includes:
  another:
    taskfile: ./path/to/another.yaml
```

### Task

| Attribute       | Type                                       | Default                                               | Description                                                                                                                                          |
| --------------- | ------------------------------------------ | ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `desc`          | `String`                                   |                                                       | Short description of the task.                                                                                                                       |
| `summary`       | `String`                                   |                                                       | Longer description of the task.                                                                                                                      |
| `sources`       | `Array<String>`                            |                                                       | Files to check for changes before running the task. Relevant for `checksum` and `timestamp` dependency resolution methods. File paths or star globs. |
| `dir`           | `String`                                   |                                                       | Directory the task should run in.                                                                                                                    |
| `method`        | `String`                                   | `checksum`                                            | Dependency method used by this task. Default to the one declared globally or `checksum`. Options: `checksum`, `timestamp` and `none`                 |
| `silent`        | `Boolean`                                  | `false`                                               | Skips some output for this task. Note that `STDOUT` and `STDERR` of the commands will still be redirected.                                           |
| `run`           | `String`                                   | The one declared globally in the Taskfile or `always` | Whether the task should run again or not if called more than once. Options: `always`, `once` and `when_changed`.                                     |
| `prefix`        | `String`                                   |                                                       | Override the prefix printed before the `STDOUT`. Only relevant when using `prefixed` output mode in the parent `Taskfile`.                           |
| `ignore_error`  | `Boolean`                                  | `false`                                               | Continue execution if errors occur while executing the commands.                                                                                     |
| `generates`     | `Array<String>`                            |                                                       | Files generated by this task. Relevant for `timestamp` dependency method. File paths or star globs.                                                  |
| `status`        | `Array<String>`                            |                                                       | Commands to check if this task should run. The task is skipped otherwise. This overrides `method`, `sources` and `generates`.                        |
| `preconditions` | `Array<`[`Precondition`](#precondition)`>` |                                                       | Commands to check if this task should run. The task errors otherwise.                                                                                |
| `vars`          | `Map<String,`[`Variable`](#variable)`>`    |                                                       | Task template variables.                                                                                                                             |
| `env`           | `Map<String,`[`Variable`](#variable)`>`    |                                                       | Task environment variables.                                                                                                                          |
| `deps`          | `Array<`[`Dependency`](#dependency)`>`     |                                                       | Dependencies of this task.                                                                                                                           |
| `cmds`          | `Array<`[`Command`](#command)`>`           |                                                       | Commands to be executed.                                                                                                                             |

You can define a `Task` using only a string for the `cmds` attribute. For example,

```yaml
tasks:
  task-a: echo "This is task A"
```

is equivalent to,

```yaml
tasks:
  task-a:
    cmds:
      - echo "This is task A"
```

You can also define a `Task` using an array of strings for the `cmds` attribute. For example,

```yaml
tasks:
  task-a:
    - echo "This is command 1 of task A"
    - echo "This is command 2 of task A"
```

is equivalent to,

```yaml
tasks:
  task-a:
    cmds:
      - echo "This is command 1 of task A"
      - echo "This is command 2 of task A"
```

### Dependency

| Attribute | Type                                    | Default | Description                                                   |
| --------- | --------------------------------------- | ------- | ------------------------------------------------------------- |
| `task`    | `String`                                |         | Name of the task to be execute as a dependency.               |
| `vars`    | `Map<String,`[`Variable`](#variable)`>` |         | Any additional variables to be passed to the referenced task. |

You can define a `Dependency` using just the name of the task. For example,

```yaml
tasks:
  task-a: echo "This is task A"
  task-b: echo "This is task B"
  task-c:
    deps: [task-a, task-b]
```

### Command

| Attribute      | Type                                    | Default | Description                                                                                                     |
| -------------- | --------------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------- |
| `cmd`          | `String`                                |         | The shell command to be executed.                                                                               |
| `defer`        | `String`                                |         | Alternative to `cmd`. Schedules the command to be executed at the end of this task instead of immediately.      |
| `silent`       | `Boolean`                               | `false` | Skips some output for this command. Note that `STDOUT` and `STDERR` of the commands will still be redirected.   |
| `ignore_error` | `Boolean`                               | `false` | Continue execution if errors happen while executing the command.                                                |
| `task`         | `String`                                |         | Alternative to `cmd`. Set this to trigger execution of another task instead of running a command.               |
| `vars`         | `Map<String,`[`Variable`](#variable)`>` |         | Any additional variables to be passed to the referenced task. Only relevant when using `task` instead of `cmd`. |

When you do not need to specify any other options, you can define a `Command` using only a string for the `cmd` attribute. For example,

```yaml
tasks:
  task-a:
    - echo "This is command 1"
    - cmd: echo "This is command 2"
```

is equivalent to,

```yaml
tasks:
  task-a:
    - cmd: echo "This is command 1"
    - cmd: echo "This is command 2"
```

### Variable

| Attribute | Type     | Default | Description                                                                    |
| --------- | -------- | ------- | ------------------------------------------------------------------------------ |
| _itself_  | `String` |         | A static value that will be set to the variable.                               |
| `sh`      | `String` |         | A shell command. The `STDOUT` of the command will be assigned to the variable. |

Variables can by static or dynamic variables. They use different syntax e.g.

```yaml
vars:
  STATIC: static
  DYNAMIC:
    sh: echo "dynamic"
```

### Precondition

| Attribute | Type     | Default | Description                                                                                                  |
| --------- | -------- | ------- | ------------------------------------------------------------------------------------------------------------ |
| `sh`      | `String` |         | Command to be executed. If a non-zero exit code is returned, the task errors without executing its commands. |
| `msg`     | `String` |         | Optional message to print if the precondition isn't met.                                                     |

If you don't need to specify a custom message, you can declare a precondition using only a string for the `sh` attribute. For example,

```yaml
tasks:
  analyze-data:
    precondition: test -f data.csv
```

is equivalent to,

```yaml
tasks:
  analyze-data:
    precondition:
      sh: test -f data.csv
```
