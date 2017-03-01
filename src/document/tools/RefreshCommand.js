import { Command } from 'substance'

class RefreshCommand extends Command {

  getCommandState (props, context) {
    return {
      disabled: false,
      active: false
    }
  }

  execute (props, context) {
    var doc = context.doc
    var annotations = doc.getIndex('annotations')
    var refresh = function (node) {
      if (node.refresh) {
        node.refresh()
      }
      if (node.hasChildren()) {
        node.getChildren().forEach(function (child) {
          refresh(child)
        })
      } else if (node.isText()) {
        annotations.get(node.getTextPath()).forEach(function (child) {
          refresh(child)
        })
      }
    }
    refresh(doc.get('content'))
    return true
  }

}

export default RefreshCommand
