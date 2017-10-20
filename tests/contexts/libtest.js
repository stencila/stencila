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
    <param name="param1" type="number">
      <default type="string">Hello!</default>
    </param>
  </params>
  <implems>
    <implem language="js" />
  </implems>
</function>
`

function one_param_with_default(param1) {
  return param1
}


export const libtestXML = `
<!DOCTYPE function PUBLIC "StencilaFunctionLibrary 1.0" "StencilaFunctionLibrary.dtd">
<library name="test">
${no_paramsXML}
${one_paramXML}
${one_param_with_defaultXML}
</library>
`

export const libtest = {
  no_params,
  one_param,
  one_param_with_default
}
