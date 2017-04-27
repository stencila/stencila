import { isFunction, DefaultDOMElement, EditorSession } from 'substance'
import { DocumentConfigurator, documentConversion, Host } from '../index.es'
import TestBackend from './backend/TestBackend'


export function spy(self, name) {
  var f
  if (arguments.length === 0) {
    f = function() {}
  }
  else if (arguments.length === 1 && isFunction(arguments[0])) {
    f = arguments[0]
  }
  else {
    f = self[name]
  }
  function spyFunction() {
    var res = f.apply(self, arguments)
    spyFunction.callCount++
    spyFunction.args = arguments
    return res
  }
  spyFunction.callCount = 0
  spyFunction.args = null
  spyFunction.restore = function() {
    if (self) {
      self[name] = f
    }
  }
  spyFunction.reset = function() {
    spyFunction.callCount = 0
    spyFunction.args = null
  }
  if (self) {
    self[name] = spyFunction
  }
  return spyFunction
}

export function wait(ms) {
  return () => {
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve()
      }, ms)
    })
  }
}

export function getSandbox(t) {
  // when running with substance-test we get
  // a sandbox for each test
  if (t.sandbox) return t.sandbox
  // otherwise we create our own DOM
  let htmlDoc = DefaultDOMElement.parseHTML('<html><body></body></html>')
  return htmlDoc.find('body')
}

export function setupEditorSession(documentId) {
  let configurator = new DocumentConfigurator()
  let docHTML
  if (!documentId) {
    docHTML = ''
  } else {
    let backend = new TestBackend()
    const entry = backend._getEntry(documentId)
    docHTML = entry.content
  }
  let doc = documentConversion.importHTML(docHTML)
  let editorSession = new EditorSession(doc, {
    configurator: configurator,
    context: {
      host: new Host({ discover: false })
    }
  })
  return {editorSession, doc}
}
