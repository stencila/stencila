import { LabelType, NodeType } from '@stencila/types'

import { Entity } from '../../../../nodes/entity'

import './caption-label'

/**
 * Finds a 'caption' slot from a node's light dom and prepends
 * the label to the first 'content' element of the captions
 */
export const createCaptionLabel = <
  T extends Entity & { label?: string; labelType?: LabelType },
>(
  node: T,
  type: NodeType
) => {
  const captions = node.querySelector('[slot="caption"]')
  if (captions) {
    // find the 'content' element of the first child node in the caption
    const firstChildContent = captions.querySelector(
      '*:first-child [slot="content"]'
    )

    // create label element and set attributes
    const labelEl = document.createElement('stencila-ui-node-caption-label')
    labelEl.setAttribute('type', type)
    labelEl.setAttribute('label-type', node.labelType)
    labelEl.setAttribute('label', node.label)

    // prepend the label to the content
    firstChildContent.prepend(labelEl)
  }
}
