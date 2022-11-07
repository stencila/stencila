import { Plugin } from 'prosemirror-state'
import { Decoration, DecorationSet } from 'prosemirror-view'

/**
 * Plugin to add placeholder text if no content in document
 *
 * From https://discuss.prosemirror.net/t/how-to-input-like-placeholder-behavior/705/13
 */
export function placeholder(text: string = 'Add content') {
  return new Plugin({
    props: {
      decorations(state) {
        const doc = state.doc

        if (
          doc.childCount > 1 ||
          !doc.firstChild?.isTextblock ||
          doc.firstChild?.content.size > 0
        )
          return

        const placeHolder = document.createElement('div')
        placeHolder.classList.add('placeholder')
        placeHolder.textContent = text

        return DecorationSet.create(doc, [Decoration.widget(1, placeHolder)])
      },
    },
  })
}
