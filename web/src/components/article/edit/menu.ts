import { EditorState, Plugin } from 'prosemirror-state'
import { EditorView } from 'prosemirror-view'

type MItem = {
  command: (
    state: EditorState<typeof articleSchema>,
    dispatch?: EditorView['dispatch'],
    view?: EditorView
  ) => unknown
  dom: HTMLElement
}

class MenuView {
  items: MItem[]
  dom: HTMLElement
  editorView: EditorView

  constructor(items: MItem[], editorView: EditorView) {
    this.items = items
    this.editorView = editorView

    this.dom = document.createElement('div')
    this.dom.className = 'menubar'
    items.forEach(({ dom }) => this.dom.appendChild(dom))
    this.update()

    this.dom.addEventListener('mousedown', (e) => {
      e.preventDefault()
      editorView.focus()
      items.forEach(({ command, dom }) => {
        if (dom.contains(e.target as HTMLElement))
          command(editorView.state, editorView.dispatch, editorView)
      })
    })
  }

  update() {
    this.items.forEach(({ command, dom }) => {
      let active = command(this.editorView.state, undefined, this.editorView)
      dom.style.display = active ? '' : 'none'
    })
  }

  destroy() {
    this.dom.remove()
  }
}

const menuPlugin = (items: MItem[]) => {
  return new Plugin({
    view(editorView) {
      const menuView = new MenuView(items, editorView)
      editorView.dom.parentNode?.insertBefore(menuView.dom, editorView.dom)
      return menuView
    },
  })
}

import { toggleMark, setBlockType, wrapIn } from 'prosemirror-commands'
import { articleSchema } from './schema'

// Helper function to create menu icons
const icon = (text: string, name: string) => {
  const span = document.createElement('stencila-icon')
  // @ts-ignore
  span.icon = 'key'

  // @ts-ignore
  span.iconOnly = true
  span.title = name
  span.textContent = text
  return span
}

// Create an icon for a heading at the given level
const heading = (level: number) => {
  return {
    command: setBlockType(articleSchema.nodes.Heading, { level }),
    dom: icon('H' + level, 'heading'),
  }
}

export const editorMenuPlugin = menuPlugin([
  { command: toggleMark(articleSchema.marks.Strong), dom: icon('B', 'strong') },
  { command: toggleMark(articleSchema.marks.Emphasis), dom: icon('i', 'em') },
  {
    command: setBlockType(articleSchema.nodes.Paragraph),
    dom: icon('p', 'paragraph'),
  },
  heading(1),
  heading(2),
  heading(3),
  {
    command: wrapIn(articleSchema.nodes.QuoteBlock),
    dom: icon('>', 'blockquote'),
  },
])
