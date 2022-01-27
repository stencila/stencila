// Type annotated declarations / assignments
var var1: object
var var2: number
let var3: string
// Use the declared type in preference to the value's type
const var4: Array<any> = 4
export const var5: Record<string, any> = 5

// Static type inference based on literals, as in `js` code analysis still applies
const var6 = true
const var7 = false
const var8 = 42
const var9 = 3.14
const var10 = 'string'
const var11 = []
const var12 = {}

function fun1(): void {}
const fun2 = (): void => {}
export function fun3(): void {}
