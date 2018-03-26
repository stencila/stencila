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
${multiplyXML}
${no_paramsXML}
${one_paramXML}
${one_param_with_defaultXML}
</library>
`

export const libtest = {
  add,
  multiply,
  no_params,
  one_param,
  one_param_with_default
}
