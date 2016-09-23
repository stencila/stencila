import AnnotationTool from 'substance/ui/AnnotationTool'

function MarkTool () {
  MarkTool.super.apply(this, arguments)
}

MarkTool.Prototype = function () {
  this.getClassNames = function () {
    return 'sc-mark-tool'
  }
}

AnnotationTool.extend(MarkTool)

export default MarkTool
