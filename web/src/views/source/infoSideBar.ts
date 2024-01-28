import { Extension } from '@codemirror/state'
import { ViewPlugin, EditorView, ViewUpdate } from '@codemirror/view'

import { SourceView } from '../source'

/**
 * Returns a codemirror `Extension` which creates a new sidebar Element
 * which should be inserted to the right of the condemirror content,
 *
 * This Bar should show the info box of the node where the cursor is present
 *
 * @param sourceView `SourceView`
 * @returns `Extension` for codemirror
 */
const infoSideBar = (sourceView: SourceView): Extension => {
  const infoSideBarPlugin = ViewPlugin.fromClass(
    class {
      /**
       * the side bar container element
       */
      dom: HTMLElement
      /**
       * the current info box element displayed
       */
      currentInfoBox: HTMLElement = null

      /**
       * id of the currently cloned node for the info box
       */
      currentNodeId: string

      /**
       * the Y coordinate of the cursor (relative to the viewport)
       */
      cursorY: number

      constructor(readonly view: EditorView) {
        this.dom = document.createElement('div')

        // this class has no functionality at this point
        // but may be needed for selecting
        this.dom.className = 'cm-stencila-info-bar'

        this.dom.style.width = '33%'
        this.dom.style.maxWidth = '300px'
        this.dom.style.minHeight = `${view.contentHeight / view.scaleY}px`
        this.view.scrollDOM.appendChild(this.dom)
      }

      update = (update: ViewUpdate) => {
        const { view: currentView } = update

        // update height of dom
        this.dom.style.minHeight = `${currentView.contentHeight / currentView.scaleY}px`

        const cursor = currentView.state.selection.main.head

        // need to handle this better
        const currentNode = sourceView
          .getNodesAt(cursor)
          .filter((node) => node.nodeType !== 'Text')[0]

        if (!currentNode) {
          return
        }

        const { nodeId } = currentNode
        // if cursor is in new node, create the new el
        if (nodeId !== this.currentNodeId) {
          this.currentNodeId = nodeId
          if (this.currentInfoBox) {
            this.currentInfoBox.remove()
          }
          const domNode = sourceView.domElement.value.querySelector(
            `#${nodeId}`
          )
          if (!domNode) {
            return
          }

          this.currentInfoBox = sourceView.domElement.value
            .querySelector(`#${nodeId}`)
            .cloneNode(true) as HTMLElement

          this.currentInfoBox.setAttribute('id', `info-box-${nodeId}`)
          this.currentInfoBox.style.position = 'absolute'
          this.dom.appendChild(this.currentInfoBox)
        }

        // reposition the infobox on the y-axis
        currentView.requestMeasure({
          read: (view) => {
            if (this.currentInfoBox) {
              const { top } = view.coordsAtPos(cursor)

              // skip if cursor y position hasn't changed
              if (this.cursorY === top) {
                return
              }
              this.cursorY = top
              const editorTop = view.scrollDOM.getBoundingClientRect().top
              const yOffset =
                this.currentInfoBox.getBoundingClientRect().height / 2
              let yPos = top - editorTop - yOffset

              if (yPos < 0) {
                yPos = 0
              }

              this.currentInfoBox.style.top = `${yPos}px`
            }
          },
        })
      }

      destroy() {
        if (this.currentInfoBox) {
          this.currentInfoBox.remove()
        }
        this.dom.remove()
      }
    }
  )

  return infoSideBarPlugin
}

export { infoSideBar }
