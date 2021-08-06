var1 = open("path1")  # no mode, assumed to be read

with open("path2", "r") as ignore:  # open with ContextManager and mode
    ignore.read()

open("path3", "r+")  # read and write, include file

open(file="path4")  # kwargs testing

open("path5", buffering=None, mode="r")

open(file="path6", mode="r", buffering=None)

open(file="path7", mode="r+")


def open_func():
    open("path8")


# The following should be detected as writes

open("write", "w")
open("write", mode="w")
open(file="write", mode="a")

# The following should be ignored

var2 = None
open(var2)
open(var2.y)
open(var2["y"])

mode = None
open("ignore", mode)
