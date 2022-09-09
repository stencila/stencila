This folder contains example of files with parameterized paths. For simplicity, they all use `calc` as the programming languages so do not
rely on external runtimes.

Try running these documents from the command line. For example, to run the file `$a.times.$b.md`:

```console
$ stencila docs run 6/times/7
```

Which is equivalent to,

```console
$ stencila docs run $a.times.$b.md a=6 b=7
```
