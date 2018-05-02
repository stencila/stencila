import { forEach, isArray } from 'substance'

const addXML = `
<function>
  <name>add</name>
  <params>
    <param name="value" type="number" />
    <param name="other" type="number" />
  </params>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

function add(value, other) {
  return value + other
}

const sumXML = `
<function>
  <name>sum</name>
  <params>
    <param name="a" type="number" />
    <param name="b" type="number" />
    <param name="c" type="number" />
    <param name="d" type="number" />
    <param name="e" type="number" />
  </params>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

function sum(...vals) {
  return vals.reduce((a,b) => {
    if (b.type === 'table') {
      forEach(b.data, (vals) => {
        a += sum(vals)
      })
      return a
    } else if (isArray(b)) {
      return sum(...b)
    } else if (isArray(b.data)) {
      return sum(...b.data)
    } else {
      return a+b
    }
  }, 0)
}

const multiplyXML = `
<function>
  <name>multiply</name>
  <params>
    <param name="value" type="number" />
    <param name="other" type="number" />
  </params>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

function multiply(value, other) {
  return value * other
}

const randXML = `
<function>
  <name>rand</name>
  <params>
  </params>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

let RAND_COUNT = 1

function rand() {
  // very pseudo random
  return RAND_COUNT++
}

// used in tests to reset the pseudo random generator
export function _reset_rand() {
  RAND_COUNT = 1
}

const no_paramsXML = `
<function>
  <name>no_params</name>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

function no_params() {
  return 5
}


const one_paramXML = `
<function>
  <name>one_param</name>
  <params>
    <param name="param1" type="number" />
  </params>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

function one_param(param1) {
  return param1 * 1.1
}


const one_param_with_defaultXML = `
<function>
  <name>one_param_with_default</name>
  <params>
    <param name="param1" type="string">
      <default type="string">Hello!</default>
    </param>
  </params>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

function one_param_with_default(param1='Hello!') {
  return param1
}


export const libtestXML = `
<!DOCTYPE function PUBLIC "StencilaFunctionLibrary 1.0" "StencilaFunctionLibrary.dtd">
<library name="test">
${addXML}
${sumXML}
${multiplyXML}
${randXML}
${no_paramsXML}
${one_paramXML}
${one_param_with_defaultXML}
</library>
`

export const libtest = {
  add,
  sum,
  multiply,
  rand,
  no_params,
  one_param,
  one_param_with_default,
}
