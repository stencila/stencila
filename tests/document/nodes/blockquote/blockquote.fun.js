import path from 'path'
import test from 'tape'
import Nightmare from 'nightmare'
var realMouse = require('nightmare-real-mouse')
realMouse(Nightmare)

test('Blockquote behaviour', function (t) {
  Nightmare({
    typeInterval: 10,
    gotoTimeout: 60000
  })
    .goto('file://' + path.resolve('build/tests/document/nodes/blockquote.html') + '?edit=1')
    .wait('.sc-visual-editor')
    .realClick('[data-id="blockquote-2"]')
    .type('[data-id="blockquote-2"]', 'in the ignorance of experts')
    .wait(100)
    .evaluate(function () {
      return document.querySelector('[data-id="blockquote-2"]').innerText.trim()
    })
    .end()
    .then(function (result) {
      t.equal(result, 'Science is the belief...in the ignorance of experts')
      t.end()
    })
    .catch(function (error) {
      t.notOk(error)
      t.end()
    })
})
