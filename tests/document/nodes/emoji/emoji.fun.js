import path from 'path'
import test from 'tape'
import Nightmare from 'nightmare'
var realMouse = require('nightmare-real-mouse')
realMouse(Nightmare)

test.skip('Emoji behaviour', function (t) {
  Nightmare({
    typeInterval: 10,
    gotoTimeout: 60000
  })
    .goto('file://' + path.resolve('build/tests/document/nodes/emoji.html') + '?edit=1')
    .wait('.sc-visual-editor')
    .realClick('[data-id="paragraph-5"]')
    .type('[data-id="paragraph-5"]', 'A :rocket: to the moon')
    .wait(100)
    .evaluate(function () {
      var code = document.querySelector('[data-id="paragraph-5"] .sc-code')
      var emoji = document.querySelector('[data-id="paragraph-5"] .sc-emoji img.emojione')
      return {
        codeText: code.innerText,
        emojiSrc: emoji.src
      }
    })
    .end()
    .then(function (result) {
      t.equal(result.codeText, ':rocket:')
      t.ok(result.emojiSrc.search('/emojione/png/1f680.png'))
      t.end()
    })
    .catch(function (error) {
      t.notOk(error)
      t.end()
    })
})
