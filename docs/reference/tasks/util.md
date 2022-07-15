<!-- Generated from Taskfile. Do not edit. -->

# `util`: Utility tasks

These are mostly very simple tasks, presently mostly used for testing.

## Tasks

### <a id='sleep'>`sleep`</a> : Sleep for a specified number of seconds

#### Command

```sh
sleep {{.SECONDS | default 1}}
```

### <a id='print-date'>`print-date`</a> : Print the current date/time

#### Command

```sh
echo $(date)
```
