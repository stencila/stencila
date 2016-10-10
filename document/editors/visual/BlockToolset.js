import Component from 'substance/ui/Component'
import switchTextType from 'substance/model/transform/switchTextType'
import deleteNode from 'substance/model/transform/deleteNode'

import BlockTool from '../../ui/BlockTool'
import HeadingTool from '../../nodes/heading/HeadingTool'
import ImageTool from '../../nodes/image/ImageTool'
import CodeblockTool from '../../nodes/codeblock/CodeblockTool'
import DefaultTool from '../../nodes/default/DefaultTool'

class BlockToolset extends Component {

  constructor (...args) {
    super(...args)

    this.primaryTypes = [
      'heading', 'paragraph', 'image', 'blockquote', 'codeblock',
      'execute'
    ]

    this.secondaryTypes = [
      'title', 'summary', 'default'
    ]

    this.tools = {
      'heading': HeadingTool,
      'image': ImageTool,
      'codeblock': CodeblockTool,
      'default': DefaultTool
    }
  }

  /**
   * Method override for custom display state
   */
  getInitialState () {
    return {
      expanded: false,
      extended: false
    }
  }

  render ($$) {
    var el = $$('div')
      .addClass('sc-toolset sc-block-toolset')

    var selected = this._getSelection()

    // CHECK
    // From a performance perspective is it better to render
    // the entire element even if it is not visible, or do this
    // and just have an empty element?
    if (!selected.type) return el

    el.addClass('sm-enabled')

    if (this.state.expanded) el.addClass('sm-expanded')

    this.primaryTypes.forEach(function (type) {
      this._addTool(selected, type, el, $$)
    }.bind(this))

    if (this.state.extended) el.addClass('sm-extended')
    el.append(
      $$('div')
        .addClass('se-tool se-extend')
        .append(
          $$('button')
            .append(
              $$('i')
                .addClass(this.state.extended ? 'fa fa-chevron-left' : 'fa fa-ellipsis-h')
            )
            .on('click', function () {
              this.extendState({
                extended: !this.state.extended
              })
            }.bind(this))
        )
    )

    var extension = $$('div')
      .ref('extension')
      .addClass('se-extension')
    this.secondaryTypes.forEach(function (type) {
      this._addTool(selected, type, extension, $$)
    }.bind(this))

    el.append(
      extension
    )

    return el
  }

  _getSelection () {
    // CHECK
    // There is more than one way to get the current selection and document, including
    // via `this.context.documentSession`. Is geeting thes via `surface` the best way?
    var surface = this.context.surfaceManager.getFocusedSurface()
    if (!surface) return {}
    var document = surface.getDocument()
    var selection = surface.getSelection()

    var enabled = false
    var type = null
    var node = null
    if (selection.isContainerSelection()) {
      // Container selections are selections over
      // multiple blocks, so don't enable
      enabled = false
    } else if (selection.isNodeSelection() || selection.isPropertySelection()) {
      if (selection.isPropertySelection()) {
        // A selection which is bound to a property (e.g. the content of a paragraph)
        // Only enable if the selection is zero length and at the start of the text
        if (selection.getStartOffset() === 0 && selection.getEndOffset() === 0) {
          enabled = true
        }
      } else {
        enabled = true
      }
      if (enabled) {
        node = document.get(
          selection.getNodeId()
        )
        if (node) {
          type = node.type
        }
      }
    }
    return {
      selection: selection,
      type: type,
      node: node
    }
  }

  _addTool (selected, type, el, $$) {
    var ToolClass = this.tools[type] || BlockTool
    var active = selected.type === type
    var disabled = !active && !this._canChange(selected, type)
    var tool = $$(ToolClass, {
      toolset: this,
      name: type,
      icon: type,
      disabled: disabled,
      active: active,
      node: selected.node
    }).ref(type + 'Tool')
    el.append(
      tool
    )
  }

  /**
   * Can the selected node be changed to the specified type
   *
   * @param      {<type>}   selected  The selected
   * @param      {<type>}   type      The type
   * @return     {boolean}  True if able to change, False otherwise.
   */
  _canChange (selected, type) {
    var node = selected.node
    var schema = this.context.doc.getSchema()
    if (node.isText()) {
      if (schema.isInstanceOf(type, 'text')) {
        // Allow `switchTextType`
        return true
      } else if (node.getText().length === 0) {
        // If empty allow replacement
        return true
      }
    }
    return false
  }

  /**
   * Change the type of the currently selected
   * node
   *
   * @param      {<type>}  type    The type
   */
  changeType (type) {
    // CHECK
    // This method is analgous to a `Command.execute` method.
    // Here, instead of having a separate command, we have just integrated it
    // into the component? What is the advantage of having a separate Command?
    var selected = this._getSelection()
    var surface = this.context.surfaceManager.getFocusedSurface()
    var selection = surface.getSelection()
    var schema = surface.getDocument().getSchema()
    surface.transaction(function (tx, args) {
      if (selected.node.isInstanceOf('text') && schema.isInstanceOf(type, 'text')) {
        // Can do a plain `switchTextType`
        args.data = {
          type: type
        }
        return switchTextType(tx, args)
      } else {
        // Do a node replacement
        // This is similar to `substance/model/transform/switchTextType` but does
        // not rquire text nodes and does not transfer annotations.
        var nodeId = selected.node.id

        // Create the new node
        var newNode = tx.create({
          type: type
        })

        // Hide the old node, show the new node
        var container = tx.get(args.containerId)
        var pos = container.getPosition(nodeId)
        if (pos >= 0) {
          container.hide(nodeId)
          container.show(newNode.id, pos)
        }

        // Delete the old node
        deleteNode(tx, { nodeId: nodeId })

        // Select the new node
        args.selection = tx.createSelection([newNode.id], selection.startOffset, selection.endOffset)
        args.node = newNode

        return args
      }
    })
  }
}

export default BlockToolset
