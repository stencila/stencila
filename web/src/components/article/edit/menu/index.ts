import { Command, setBlockType, toggleMark, wrapIn } from 'prosemirror-commands'
import { Plugin } from 'prosemirror-state'
import { EditorView } from 'prosemirror-view'
import { menuButton } from './icon'
import { articleSchema } from '../schema'

type MItem = {
  command: Command<typeof articleSchema>
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
    this.dom.style.position = 'sticky'
    this.dom.style.background = 'var(--color-stock)'
    this.dom.style.zIndex = '2'
    this.dom.style.borderBottom = '1px solid var(--color-neutral-300)'
    this.dom.style.top = '0'
    this.dom.style.left = '0'

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
      const active = command(this.editorView.state, undefined, this.editorView)
      if (active) {
        dom.removeAttribute('disabled')
        dom.classList.add('active')
      } else {
        dom.setAttribute('disabled', 'disabled')
        dom.classList.remove('active')
      }
    })
  }

  destroy() {
    this.dom.remove()
  }
}

const menuPlugin = (items: MItem[]): Plugin => {
  return new Plugin({
    view(editorView) {
      const menuView = new MenuView(items, editorView)
      editorView.dom.parentNode?.insertBefore(menuView.dom, editorView.dom)
      return menuView
    },
  })
}

// Create an icon for a heading at the given level
const heading = (level: number): MItem => {
  return {
    command: setBlockType(articleSchema.nodes.Heading, { depth: level }),
    dom: menuButton(`H${level}`, 'heading'),
  }
}

export const editorMenuPlugin = menuPlugin([
  {
    command: toggleMark(articleSchema.marks.Strong),
    dom: menuButton('B', 'strong'),
  },
  {
    command: toggleMark(articleSchema.marks.Emphasis),
    dom: menuButton('i', 'em'),
  },
  {
    command: setBlockType(articleSchema.nodes.Paragraph),
    dom: menuButton('p', 'paragraph'),
  },
  heading(1),
  heading(2),
  heading(3),
  {
    command: wrapIn(articleSchema.nodes.QuoteBlock),
    dom: menuButton('>', 'blockquote'),
  },
  {
    command: setBlockType(articleSchema.nodes.CodeBlock),
    dom: menuButton('CodeBlock', 'codeBlock'),
  },
])
