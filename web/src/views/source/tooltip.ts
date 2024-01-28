import { hoverTooltip } from '@codemirror/view'

import { SourceView } from '../source'

/**
 * Create a tooltip on hover for the source codemirror `Extension`
 * @param sourceView instance of the current `SourceView`
 * @returns `Extension`
 */
const tooltipOnHover = (sourceView: SourceView) =>
  hoverTooltip(
    (_, pos: number) => {
      return {
        pos,
        above: true,
        create: () => {
          // fetch id of hovered node, filter out 'Text' nodes
          const { nodeId } = sourceView
            .getNodesAt(pos)
            .filter((node) => node.nodeType !== 'Text')[0]

          // create clone of node from the `DomClient`
          const domNode = sourceView.domElement.value.querySelector(
            `#${nodeId}`
          )

          if (domNode) {
            const dom = domNode.cloneNode(true) as HTMLElement
            // change id to avoid duplicates
            dom.setAttribute('id', `tooltip-${nodeId}`)
            return { dom, offset: { x: 10, y: 10 } }
          }
        },
      }
    },
    { hoverTime: 500 }
  )

export { tooltipOnHover }
