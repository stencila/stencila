import test from 'tape'
import { isNil } from 'substance'
import { getSandbox, setupEditorSession } from '../testHelpers'
import InlineCellComponent from '../../src/document/nodes/inline-cell/InlineCellComponent'

test('InlineCellComponent: Mounting a InlineCellComponent', (t) => {
  const sandbox = getSandbox(t)
  const {editorSession, doc} = setupEditorSession()
  doc.create({
    type: 'paragraph',
    id: 'p1',
    content: 'abcdefg'
  })
  let cell = doc.create({
    type: 'inline-cell',
    path: ['p1', 'content'],
    startOffset: 3,
    endOffset: 4,
    expression: '"FOO"'
  })
  InlineCellComponent.mount({
    node: cell
  }, sandbox)
  let el = sandbox.find('.sc-inline-cell')
  t.notOk(isNil(el), 'There should be a .sc-inline-cell.')
  // Without a CellEngine we need to trigger evaluation manually
  cell.recompute()
  // This does only work if the event listeners are attached correctly
  t.equal(el.textContent, 'FOO', 'The value should have been rendered.')
  t.end()
})
