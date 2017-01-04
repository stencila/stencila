import path from 'path'
import test from 'tape'
import Nightmare from 'nightmare'
require('nightmare-real-mouse')(Nightmare)

test.skip('Add to an existing discussion', function (t) {
  Nightmare({
    typeInterval: 10,
    gotoTimeout: 60000
  })
    .goto('file://' + path.resolve('build/tests/document/nodes/discussion.html') + '?edit=1')
    .wait('.sc-visual-editor')
    // Click on mark
    .realClick('.sc-visual-editor [data-id="mark-test-1"]')
    // Click on add comment button
    .wait(100)
    .realClick('.sc-visual-editor [data-id="discussion-test-1"] .se-add')
    // Type into the new comment
    .wait(100)
    .realClick('.sc-visual-editor [data-id="discussion-test-1"] .sc-comment:last-child .sc-paragraph')
    .type('.sc-visual-editor [data-id="discussion-test-1"] .sc-comment:last-child .sc-paragraph', 'One, one, one')
    // Get the typed text
    .wait(100)
    .evaluate(function () {
      return document.querySelector('.sc-visual-editor [data-id="discussion-test-1"] .sc-comment:last-child .sc-paragraph').innerText.trim()
    })
    .end()
    .then(function (result) {
      t.equal(result, 'One, one, one')
      t.end()
    })
    .catch(function (error) {
      t.notOk(error)
      t.end()
    })
})

test.skip('Create a new discussion', function (t) {
  Nightmare({
    typeInterval: 10,
    gotoTimeout: 60000
  })
    .goto('file://' + path.resolve('build/tests/document/nodes/discussion.html') + '?edit=1')
    .wait('.sc-visual-editor')
    // Double click to select some test and get inline toolset
    .realClick('.sc-visual-editor [data-id="paragraph-test-2"] [data-id="strong-test-1"]')
    .realClick('.sc-visual-editor [data-id="paragraph-test-2"] [data-id="strong-test-1"]')
    // Click on mark tool to create a new discussion
    .wait(100)
    .realClick('.sc-mark-tool')
    // Type into the new paragraph
    .wait(100)
    // FIXME: currently it is necessary to click twice on the new paragraph
    .realClick('.sc-visual-editor .sc-discussion:last-child .sc-paragraph')
    .realClick('.sc-visual-editor .sc-discussion:last-child .sc-paragraph')
    .type('.sc-visual-editor .sc-discussion:last-child .sc-paragraph', 'Testing, 1, 2, 3')
    // Get the typed text
    .wait(100)
    .evaluate(function () {
      return document.querySelector('.sc-visual-editor .sc-discussion:last-child .sc-paragraph').innerText.trim()
    })
    .end()
    .then(function (result) {
      t.equal(result, 'Testing, 1, 2, 3')
      t.end()
    })
    .catch(function (error) {
      t.notOk(error)
      t.end()
    })
})
