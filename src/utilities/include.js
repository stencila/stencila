const visit = require('unist-util-visit')

module.exports = function include () {
  return function (ast) {
    return visit(ast, function (node) {
      if (node.type === 'paragraph') {
        node.children.forEach(function (child) {
          if (child.type === 'text' && child.value && child.indexOf('<') === 0) {
            console.log('this is an include')
          }
        })
      }
    })
  }
}
