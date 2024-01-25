<!-- Currently just some random notes --->

Unlike in v1, only one file per kernel to keep things simple.

The unicode flags used:

| Flag  | Unicode  | Hexadecimal | Purpose                                                  |
| ----- | -------- | ----------- | -------------------------------------------------------- |
| READY | U+10ACDC |             | Sent by the microkernel to signal it is ready for a task |
| EXEC  | U+10B522 |             | Execute the following code with possible side-effects    |
| EVAL  | U+1010CC |             | Evaluate the following code without side-effects         |
| FORK  | U+10DE70 |             | Execute the following code in a fork                     |
| LINE  | U+10ABBA |             | A newline in a task                                      |
| END   | U+10CB40 |             | End of an output value or message                        |

In Bash you can get the hexdecimal equivlents using `hexdump` e.g.

```console
echo -e '\U10ACDC' | hexdump -C
```

In Fish shell, use this slightly different syntax:

```console
echo \U10ACDC | hexdump -C
```
