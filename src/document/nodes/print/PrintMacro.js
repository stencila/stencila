import InlineNodeMacro from '../../ui/InlineNodeMacro'

class PrintMacro extends InlineNodeMacro {

  get regex () {
    return /((\{print\s+)|(\$\{))(.+)\}/
  }

  createNodeData (match) {
    return {
      type: 'print',
      source: match[4].trim()
    }
  }

}

export default PrintMacro
