import test from 'tape'
import ProjectTab from '../../src/project/ProjectTab'
import { addNewDocument } from '../../src/project/ProjectManipulations'
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

test('Project: add a new Sheet', t => {
  let { archive, manifest } = _setup(sample())
  let newId = addNewDocument(archive, 'sheet')
  let entry = manifest.getDocumentEntry(newId)
  t.ok(Boolean(entry), 'there should be a new entry')
  t.equal(entry.type, 'sheet', '.. of type sheet')
  t.equal(entry.name, 'Sheet2', '.. with name "Sheet2"')
  t.end()
})

test('Project: add a new Article', t => {
  let { archive, manifest } = _setup(sample())
  let newId = addNewDocument(archive, 'article')
  let entry = manifest.getDocumentEntry(newId)
  t.ok(Boolean(entry), 'there should be a new entry')
  t.equal(entry.type, 'article', '.. of type sheet')
  t.equal(entry.name, 'Article2', '.. with name "Article2"')
  t.end()
})

test('Project: rename resources with duplicate names on ingestion', t => {
  let { manifest } = _setup(duplicate())
  let entry1 = manifest.getDocumentEntry('sheet1')
  let entry2 = manifest.getDocumentEntry('sheet2')
  t.ok(entry1.name !== entry2.name, 'the name collision should have been resolved')
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

function duplicate() {
  return [
    {
      id: 'sheet1',
      path: 'sheet1.xml',
      type: 'sheet',
      name: 'Sheet1',
      cells: []
    },
    {
      id: 'sheet2',
      path: 'sheet2.xml',
      type: 'sheet',
      name: 'Sheet1',
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