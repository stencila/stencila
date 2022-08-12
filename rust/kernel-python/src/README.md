# Stencila Python Microkernel

## Development

During development it can be useful to manually test / debug the microkernel. You should be able to type lines of Python code and get back results e.g.

```console
> python3 src/python_kernel.py
READY
READY
a = 6 * 7
TASK
TASK
a
42RESULT
TASK
TASK
EXIT
```
