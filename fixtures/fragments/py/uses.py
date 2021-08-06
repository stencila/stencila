var1
var2 + var3
fun1()
fun2(var4, var5)
var6[var7]

if var8 < var9:
    ignore = var10

# Ignore the item identifier in for loops 

for ignore in var11:
    var12 * ignore

# Ignore the item identifier of with clauses

with var13 as ignore:
    ignore * ignore

# Ignore attribute identifiers

var14.ignore
var14.ignore.ignore
var14.ignore(var15)

# Ignore the names of arguments

fun3(var16, ignore=1, ignore='a')

# Ignore identifiers used in functions or function parameters

def fun4(ignore1, ignore2=1):
    ignore1 * ignore3(ignore2)

lambda ignore: ignore

# Skip identifiers that are assigned to

assign = 1
