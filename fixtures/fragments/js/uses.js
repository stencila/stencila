var1
var2 + var3
fun1()
fun2(var4, var5)
var6[var7]

if (var8 < var9) {
  ignore = var10
}
// Identifiers used to construct object

;({ a: varA, varB, c: varC })

// Identifiers used to construct array

;[varD, varE]

// Ignore identifiers that are imported

import ignore from 'fs'
import { ignore } from 'fs'

// Ignore the item identifier in for loops

for (ignore in var11) {
  var12 * ignore
}

for (let ignore = 0; ignore < var13.len; ignore++) {
  ignore * ignore
}

// Ignore property identifiers

var14.ignore
var14.ignore.ignore
var14.ignore(var15)
{
  ignore: {
    ignoredAlso: true
  }
}

// Ignore identifiers used in functions or function parameters

function func1(ignore1, ignore2 = 1) {
  ignore1 * ignore3(ignore2)
}

const func2 = (ignore) => {
  ignore
}

// Skip identifiers that are assigned to

assign = 1
var assign = 2
let assign = 3
const assign = 4
