import test from 'tape'
import Nightmare from 'nightmare'
var realMouse = require('nightmare-real-mouse')
realMouse(Nightmare)

test('Emoji behaviour', function (t) {
  Nightmare({
    typeInterval: 5
  })
    .goto('http://localhost:9000/tests/document/nodes/emoji?edit=1')
    .wait('.sc-visual-editor')
    .realClick('[data-id="paragraph-5"]')
    .type('[data-id="paragraph-5"]', 'A :rocket: to the moon')
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
