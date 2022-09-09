#!/usr/bin/env python3

import json
import os
import resource
from sys import exit, stdin, stdout, stderr

from python_codec import decode_value, encode_exception, encode_message, encode_value

# Use easier-to-type flags during development manual testing
if stdin.isatty():
    READY = "READY\n"
    RESULT = "RESULT\n"
    TASK = "TASK\n"
    FORK = "FORK\n"
    NEWLINE = "NEWLINE"
    EXIT = "EXIT\n"
else:
    READY = "\U0010ACDC\n"
    RESULT = "\U0010CB40\n"
    TASK = "\U0010ABBA\n"
    FORK = "\U0010DE70"
    NEWLINE = "\U0010B522"
    EXIT = "\U0010CC00"

# Try to get the maximum number of file descriptors the process can have open
#
# SC_OPEN_MAX "The maximum number of files that a process can have open at any time" sysconf(3)
# RLIMIT_NOFILE "specifies a value one greater than the maximum file descriptor number that can be opened by this process." getrlimit(2)
try:
    MAXFD = os.sysconf("SC_OPEN_MAX")
except Exception:
    try:
        MAXFD = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    except Exception:
        MAXFD = 256

# Monkey patch `print` to encode individual objects (if no options used)
def print(*objects, sep=" ", end="\n", file=stdout, flush=False):
    if sep != " " or end != "\n" or file != stdout or flush:
        return __builtins__.print(*objects, sep, end, file, flush)
    for object in objects:
        json = encode_value(object)
        stdout.write(json + RESULT)


# Create execution context with monkey patched `print` and `decode_value` function
# for setting variables
context = {}
context.update({"print": print, "__decode_value__": decode_value})

# Signal that kernel is ready
stdout.write(READY)
stdout.flush()
stderr.write(READY)
stderr.flush()

# The pids of forks of this kernel
fork_pids = []

while True:
    try:
        task = stdin.readline()
        lines = task.split(NEWLINE)

        if lines[0] == EXIT:
            exit()

        should_exec = True
        should_exit = False
        if lines[0] == FORK:
            pid = os.fork()
            if pid > 0:
                # Parent process, so return the pid of the fork and
                # then wait for the next task

                fork_pids.append(pid)

                stdout.write(str(pid) + RESULT)
                stdout.write(TASK)
                stdout.flush()
                stderr.write(TASK)
                stderr.flush()

                should_exec = False
            else:
                # Remove the FORK flag and the pipe paths from the front of lines
                (new_stdin, new_stdout, new_stderr) = lines[1:4]
                lines = lines[4:]

                # Close file descriptors so that we're not interfering with
                # parent's file descriptors and so stdin, stdout and stderr get replaced below
                # using the right index (0, 1, 2).
                os.closerange(0, MAXFD)

                if new_stdin:
                    # Replace stdin with pipe and do not execute code (which should be empty)
                    os.open(new_stdin, os.O_RDONLY)  # fd 0 = stdin
                    should_exec = False
                else:
                    # If there is no new stdin then set stdin to /dev/null to avoid getting more input
                    # and exit at end of loop
                    os.open("/dev/null", os.O_RDONLY)  # fd 0 = stdin
                    should_exit = True

                # Replace stdout and stderr with pipes
                os.open(new_stdout, os.O_WRONLY | os.O_TRUNC)  # fd 1 = stdout
                os.open(new_stderr, os.O_WRONLY | os.O_TRUNC)  # fd 2 = stderr

        if should_exec:
            rest, last = lines[:-1], lines[-1]
            try:
                try:
                    last = compile(last, "<code>", "eval")
                except:
                    compiled = compile("\n".join(lines), "<code>", "exec")
                    exec(compiled, context)
                else:
                    if rest:
                        joined = "\n".join(rest)
                        compiled = compile(joined, "<code>", "exec")
                        exec(compiled, context)
                    value = eval(last, context)
                    if value is not None:
                        json = encode_value(value)
                        stdout.write(json + RESULT)
            except KeyboardInterrupt as interrupt:
                stderr.write(
                    encode_message(
                        "Interrupt", "Code execution was interrupted", interrupt
                    )
                    + RESULT
                )
            except Exception as exc:
                stderr.write(encode_exception(exc) + RESULT)

            stdout.write(TASK)
            stdout.flush()
            stderr.write(TASK)
            stderr.flush()

        # Forks should exit after executing the code
        if should_exit:
            exit()

        # Parent kernel needs to call `waitpid` on forks so that
        # they do not become zombie ('defunct' processes)
        for pid in fork_pids:
            try:
                (pid, code) = os.waitpid(pid, os.WNOHANG)
                if pid != 0:
                    fork_pids.remove(pid)
            except ChildProcessError as exc:
                if exc.errno != 10:  # [Errno 10] No child processes
                    raise exc

    except KeyboardInterrupt:
        # Ignore any interrupts that get accidentally sent while there
        # is no task running to avoid ending the kernel process
        pass
