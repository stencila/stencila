const visit = require('unist-util-visit')

function md2html () {
  return function (tree) {
    visit(tree, function (node, i, parent) {})
  }
}

function html2md () {
  return function (tree) {
    visit(tree, function (node, i, parent) {})
  }
}

module.exports = {
  md2html: md2html,
  html2md: html2md
}
