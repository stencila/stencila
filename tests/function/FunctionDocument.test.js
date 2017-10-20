import test from 'tape'

import importFunctionDocument from '../../src/function/importFunctionDocument'

test('FunctionDocument: getUsage()', t => {
  let func1 = importFunctionDocument(`<function>
    <name>func1</name>
</function>`)
  t.deepEqual(func1.getUsage(), {
    name: 'func1', 
    summary: '', 
    examples: [], 
    params: [], 
    return: { type: 'any', description: '' } 
  })

  let func2 = importFunctionDocument(`<function>
    <name>func2</name>
    <summary>Function summary</summary>
    <params>
      <param name="param1" type="string">
        <default type="string">Yo</default>
        <description>Parameter one</description>
      </param>
    </params>
    <return type="string">
      <description>The return value</description>
    </return>
    <examples>
      <example>
        <description>This is an example</description>
        <usage>func2()</usage>
      </example>
      <example>
        <description>This is another example</description>
        <usage>func2(3)</usage>
      </example>
    </examples>
</function>`)
  t.deepEqual(func2.getUsage(), {
    name: 'func2', 
    summary: 'Function summary', 
    examples: [
      'func2()', 'func2(3)'
    ],
    params: [{
      name: 'param1',
      type: 'string',
      description: 'Parameter one',
      default: { 
        type: 'string', 
        data: 'Yo'
      }
    }], 
    return: {
      type: 'string', 
      description: 'The return value' 
    }
  })

  t.end()
})
