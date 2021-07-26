f = open("path1")  # no mode, assumed to be read

with open("path2", "r") as f:  # open with ContextManager and mode
    a = f.read()

open("path3", "r+")  # read and write, include file

open(file="path4")  # kwargs testing

open("path5", buffering=None, mode="r")

open(file="path6", mode="r", buffering=None)

open(file="path7", mode="r+")


def open_func():
    open("path8")


# The following should be ignored

# Write mode so ignore
open("ignore", "w")  # is a write, don't include
open("ignore", mode="w")
open(file="ignore", mode="w")

# Variable mode, don't know if read or write, so ignore
mode = None
open("ignore", mode)

# Don't know the actual file name since it's a variable, don't include
var = None
open(var)
open(var.y)
open(var["y"])
