The `bash` kernel supports outputting most primitive node types via JSON parsing of `stdout`. The custom `print` function allow separation of outputs (rather than just a single sting):

```bash exec
print true 1 2.34
print "string"
print '[1, 2, 3]'
echo '{"a": 1, "b": 2}'
```