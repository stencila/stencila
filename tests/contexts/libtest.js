const fooXML = `
<function>
  <name>foo</name>
  <params>
  </params>
  <implems>
    <implem language="js" />
  </implems>
  <tests>
  </tests>
</function>
`

function foo() {
  return 5
}

export const libtestXML = `
<!DOCTYPE function PUBLIC "StencilaFunctionLibrary 1.0" "StencilaFunctionLibrary.dtd">
<library name="test">
${fooXML}
</library>
`

export const libtest = {
  foo
}
