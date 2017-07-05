// TODO: this is a bit hacky,
// maybe we might want to be able to configure this
const CONTEXTS = ['js', 'py', 'r', 'sql']

export default function _getContextNameForExpression(expr) {
  const nodes = expr.nodes
  for (var i = 0; i < nodes.length; i++) {
    let node = nodes[i]
    if (node.type === 'call' && CONTEXTS.indexOf(node.name)>=0) {
      return node.name
    }
  }
}