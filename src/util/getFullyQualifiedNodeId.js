export default function getFullyQualifiedNodeId(node) {
  return `${node.document.UUID}#${node.id}`
}