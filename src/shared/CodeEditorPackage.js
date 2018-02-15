import { Document, TextNode, BasePackage } from 'substance'

export default {
  name: 'CodeEditor',
  configure(config) {
    // TODO this should be better reusable
    // this configurations are necessary
    config.defineSchema({
      name: 'code-editor',
      version: '1.0',
      defaultTextType: 'cell',
      DocumentClass: Document
    })
    class CellNode extends TextNode {}
    CellNode.type = 'cell'
    config.addNode(CellNode)
    config.addEditorOption({key: 'forcePlainTextPaste', value: true})
    config.import(BasePackage)
  }
}
