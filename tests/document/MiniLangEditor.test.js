import test from 'tape'
import { isNil } from 'substance'
import {
  getSandbox, setupEditorSession,
  TestEvent, ComponentWrapper, StubDomSelection
} from '../testHelpers'
import MiniLangEditor from '../../src/document/nodes/cell/MiniLangEditor'

test('MiniLangEditor: Mounting a MiniLangEditor', (t) => {
  const sandbox = getSandbox(t)
  const {editorSession, doc} = setupEditorSession()
  doc.create({
    type: 'cell',
    id: 'cell1',
    expression: 'a+b'
  })
  const path = ['cell1', 'expression']
  ComponentWrapper(MiniLangEditor, {
    props: {path},
    context: _context(editorSession)
  }).mount(sandbox)
  let el = sandbox.find('.sc-mini-lang-editor')
  t.notOk(isNil(el), 'There should be a .sc-mini-lang-editor.')
  t.end()
})

test('MiniLangEditor: Enter key handling', (t) => {
  const sandbox = getSandbox(t)
  const {editorSession, doc} = setupEditorSession()
  doc.create({
    type: 'cell',
    id: 'cell1',
    expression: 'x'
  })
  const path = ['cell1', 'expression']
  const wrapper = ComponentWrapper(MiniLangEditor, {
    props: {path},
    context: _context(editorSession)
  }).mount(sandbox)
  const editor = wrapper.refs.component
  let surface = editor.refs.contentEditor
  editorSession.setSelection({
    type: 'property',
    path: path,
    startOffset: 1,
    surfaceId: surface.id
  })
  surface._handleEnterKey(new TestEvent())
  t.equal(doc.get(path), 'x\n', 'Should have inserted a new-line')
  t.end()
})

function _context(editorSession) {
  return {
    editorSession,
    domSelection: new StubDomSelection()
  }
}