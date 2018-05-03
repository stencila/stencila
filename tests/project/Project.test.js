import test from 'tape'
import ProjectTab from '../../src/project/ProjectTab'
import createRawArchive from '../util/createRawArchive'
import loadRawArchive from '../util/loadRawArchive'
import { getSandbox } from '../testHelpers'

test('Project: validate resource name input on-the-fly', t => {
  let el = getSandbox(t)
  let { archive, manifest } = _setup(sample())
  let entry = manifest.get('sheet')
  let projectTab = new ProjectTab(null, { entry, active: true }, { context: { documentArchive: archive } })
  projectTab.mount(el)
  projectTab._editDocumentName()
  let input = projectTab.refs['documentName']
  input.val('Foo')
  input.el.emit('input')
  t.equal(projectTab.state.error, false, 'There should be no error')
  input.val("Foo's Bar")
  input.el.emit('input')
  t.ok(projectTab.state.error, 'input validator should reject tick marks')
  projectTab.extendState({ error: false })
  input.val("My Article")
  input.el.emit('input')
  t.ok(projectTab.state.error, 'input validator should reject name collisions with existing resources')
  // removing the component to dispose() and to disable things like blur handlers
  projectTab.remove()
  t.end()
})

function sample() {
  return [
    {
      id: 'article',
      path: 'article.xml',
      type: 'article',
      name: 'My Article',
      body: []
    },
    {
      id: 'sheet',
      path: 'sheet.xml',
      type: 'sheet',
      name: 'My Sheet',
      cells: []
    }
  ]
}

function _setup(archiveData) {
  let context = {}
  let rawArchive = createRawArchive(archiveData)
  let archive = loadRawArchive(rawArchive, context)
  let manifest = archive.getEditorSession('manifest').getDocument()
  return { archive, manifest }
}