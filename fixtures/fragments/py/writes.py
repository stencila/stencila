var1 = open("path1", "w")

var2 = []
with open("path2", "w+") as f: 
    f.write(var2)

open("path3", "w+") 

open(file="path4", mode="a") 

def open_func():
    open("path5", 'w')


# The following should be detected as reads

open("read", "r")
open("read", mode="r")
open(file="read", mode="r+")

# The following should be ignored

var2 = None
open(var2)
