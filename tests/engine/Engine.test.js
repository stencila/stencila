import test from 'tape'
import Engine from '../../src/engine/_Engine'

/* TODO:
  - single cell
  - exposing a variable
  - two dependent cells
  - update a cell
  - hybrid: using mini + js
*/

test('Engine: single cell', t => {
  let e = new Engine()
  t.plan(1)
  let cell = e.createCell('1+2', 'mini', '')
})
