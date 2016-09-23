import InlineNodeMacro from '../../ui/InlineNodeMacro'

function PrintMacro () {
};

PrintMacro.Prototype = function () {
  this.regex = /((\{print\s+)|(\$\{))(.+)\}/

  this.createNodeData = function (match) {
    return {
      type: 'print',
      source: match[4].trim()
    }
  }
}

InlineNodeMacro.extend(PrintMacro)

export default PrintMacro
