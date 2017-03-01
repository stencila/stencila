import { AnnotationCommand, uuid, getRelativeBoundingRect } from 'substance'
import moment from 'moment'

class MarkCommand extends AnnotationCommand {

  constructor () {
    super({
      name: 'mark',
      nodeType: 'mark'
    })
  }

  /**
   * Override `AnnotationCommand.getAnnotationData` to be able to provide
   * a `target` for the mark
   *
   * @return     {Object}  The annotation data.
   */
  getAnnotationData () {
    return {
      target: uuid('discussion')
    }
  }

  execute (props, context) {
    var result = super.execute(props, context)
    var mark = result.anno

    // Create a new `Discussion` node after the end of the current selection
    if (result.mode === 'create') {
      var surface = context.surfaceManager.getSurface(props.selection.surfaceId)
      var discussionId
      surface.transaction(function (tx, args) {
        // Create the new discussion with an initial comment
        var user = context.documentSession.config.user
        var paragraph = tx.create({
          type: 'paragraph'
        })
        var comment = tx.create({
          type: 'comment',
          who: '@' + user,
          when: moment().format(),
          nodes: [paragraph.id]
        })
        var discussion = tx.create({
          id: mark.target,
          type: 'discussion',
          nodes: [comment.id]
        })
        // Insert the new node after the current one
        var container = tx.get(args.containerId)
        var pos = container.getPosition(args.selection.getNodeId())
        container.show(discussion.id, pos + 1)

        args.node = discussion
        args.selection = tx.createSelection([paragraph.id, 'content'], 0, 0)

        // CHECK
        // There must be a better way to get the id of the new discussion back
        // from the transaction?
        discussionId = discussion.id

        return args
      })

      // CHECK
      // Hackity, hack, hack
      // Better way to do this?!!#%@#!
      var position
      var componentEl = document.querySelector('[data-id=' + mark.id + ']')
      var containerEl = context.surfaceManager.surfaces.content.parent.el.el
      if (componentEl && containerEl) {
        position = getRelativeBoundingRect(componentEl, containerEl)
      } else {
        position = {
          top: 1,
          right: 1
        }
      }
      document.dispatchEvent(new window.CustomEvent('mark:selected', {
        detail: {
          discussionId: discussionId,
          markPosition: position
        }
      }))

      return true
    }

    return false
  }
}

export default MarkCommand
