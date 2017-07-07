import test from 'tape'
import { isNil, Component, platform } from 'substance'
import { getSandbox, setupEditorSession } from '../testHelpers'
import CodeEditorComponent from '../../src/document/ui/CodeEditorComponent'

test('CodeEditorComponent: Mounting a CodeEditorComponent', (t) => {
  const sandbox = getSandbox(t)
  const {editorSession, doc} = setupEditorSession()
  doc.create({
    type: 'cell',
    id: 'cell1',
    expression: 'js()',
    sourceCode: 'return 5'
  })
  class AppStub extends Component {
    getChildContext() {
      return {
        editorSession: this.props.editorSession
      }
    }
    render($$) {
      return $$('div').append(
        $$(CodeEditorComponent, {
          path: ['cell1', 'sourceCode'],
          language: 'js'
        }).ref('editor')
      )
    }
  }
  let app = AppStub.mount({editorSession}, sandbox)
  let editor = app.refs.editor
  let el = sandbox.find('.sc-code-editor')
  t.notOk(isNil(el), 'There should be a .sc-code-editor')
  // Note: ace is only available in the browser
  if (platform.inBrowser) {
    t.equal(editor.aceEditor.getValue(), 'return 5', 'The code editor should have been set up.')
  }
  t.end()
})
