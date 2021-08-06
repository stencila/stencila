var1
fun1()
var2 + var3
fun2(var4, var5)
var6[var7]

if (var8 < var9) {
    ignore <- var10
}

# Ignore the item identifier in for loops 

for (ignore in var11) {
    var12 * ignore
}

# Ignore the names of arguments

median(var13, ignore=1, ignore='a')

# Ignore property identifiers

var14$ignore
var14$ignore$ignore
var14$ignore(var15)

# Ignore identfiers used in functions

function () {
    ignore * ignore(ignore)
}

# Skip identifiers that are assigned to

assign <- 1
assign = 2
assign <<- 3
assign <- function() {}
