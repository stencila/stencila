/** Slash-command authoring menu for ResearchObject wrappers. */
import { autoUpdate, computePosition, flip, offset, shift } from '@floating-ui/dom'
import { type Editor, Extension, type Range } from '@tiptap/core'
import { PluginKey } from '@tiptap/pm/state'
import Suggestion, { type SuggestionProps } from '@tiptap/suggestion'

import {
  RESEARCH_OBJECTS,
  type ResearchObjectNodeName,
  activeResearchObject,
  canWrapInResearchObject,
} from './research-objects'

export type SlashMenuItem = {
  title: string
  icon: string
} & (
  | { kind: 'wrap'; name: ResearchObjectNodeName }
  | { kind: 'unwrap'; name?: undefined }
)

const UNWRAP_ITEM: SlashMenuItem = {
  kind: 'unwrap',
  title: 'Unwrap',
  icon: 'i-lucide:ungroup',
}

export function filterSlashMenuItems(
  query: string,
  options: { canWrap: boolean; isWrapped: boolean }
): SlashMenuItem[] {
  const items: SlashMenuItem[] = []

  if (options.canWrap) {
    items.push(
      ...RESEARCH_OBJECTS.map(
        ({ name, title, icon }): SlashMenuItem => ({
          kind: 'wrap',
          name,
          title,
          icon,
        })
      )
    )
  }
  if (options.isWrapped) {
    items.push(UNWRAP_ITEM)
  }

  const needle = query.trim().toLowerCase()
  return needle
    ? items.filter((item) => item.title.toLowerCase().startsWith(needle))
    : items
}

function applySlashMenuItem(editor: Editor, range: Range, item: SlashMenuItem) {
  const chain = editor.chain().focus().deleteRange(range)

  if (item.kind === 'wrap') {
    chain.wrapIn(item.name).run()
    return
  }

  const active = activeResearchObject(editor)
  if (active) {
    chain.lift(active).run()
  } else {
    chain.run()
  }
}

/** Short-lived DOM view driven by Tiptap's Suggestion lifecycle. */
class SlashMenuList {
  private readonly element: HTMLDivElement
  private items: SlashMenuItem[] = []
  private selectedIndex = 0
  private props: SuggestionProps<SlashMenuItem>
  private stopFloating?: () => void

  constructor(props: SuggestionProps<SlashMenuItem>) {
    this.props = props
    this.element = document.createElement('div')
    this.element.className = 'stencila-edit-slash-menu'
    this.element.setAttribute('role', 'listbox')
    this.element.setAttribute('aria-label', 'Research object actions')
    document.body.append(this.element)
    this.update(props)
  }

  update(props: SuggestionProps<SlashMenuItem>) {
    this.props = props
    this.items = props.items
    this.selectedIndex = Math.min(
      this.selectedIndex,
      Math.max(this.items.length - 1, 0)
    )
    this.renderItems()
    this.position()
  }

  onKeyDown(event: KeyboardEvent): boolean {
    if (this.items.length === 0) {
      return false
    }

    if (event.key === 'ArrowDown') {
      this.selectedIndex = (this.selectedIndex + 1) % this.items.length
      this.renderItems()
      return true
    }
    if (event.key === 'ArrowUp') {
      this.selectedIndex =
        (this.selectedIndex + this.items.length - 1) % this.items.length
      this.renderItems()
      return true
    }
    if (event.key === 'Enter' || event.key === 'Tab') {
      this.select(this.selectedIndex)
      return true
    }

    return false
  }

  destroy() {
    this.stopFloating?.()
    this.element.remove()
  }

  private select(index: number) {
    const item = this.items[index]
    if (item) {
      this.props.command(item)
    }
  }

  private renderItems() {
    this.element.replaceChildren(
      ...this.items.map((item, index) => {
        const button = document.createElement('button')
        button.type = 'button'
        button.className = 'stencila-edit-slash-menu-item'
        button.classList.toggle('selected', index === this.selectedIndex)
        button.setAttribute('role', 'option')
        button.setAttribute('aria-selected', String(index === this.selectedIndex))

        const icon = document.createElement('span')
        icon.className = item.icon
        icon.setAttribute('aria-hidden', 'true')

        const label = document.createElement('span')
        label.textContent =
          item.kind === 'wrap' ? `Wrap in ${item.title}` : item.title

        button.append(icon, label)
        button.addEventListener('mousedown', (event) => event.preventDefault())
        button.addEventListener('click', () => this.select(index))
        return button
      })
    )
  }

  private position() {
    this.stopFloating?.()

    const clientRect = this.props.clientRect?.()
    if (!clientRect) {
      return
    }

    const reference = {
      contextElement: this.props.editor.view.dom,
      getBoundingClientRect: () => this.props.clientRect?.() ?? clientRect,
    }
    this.stopFloating = autoUpdate(reference, this.element, () => {
      void computePosition(reference, this.element, {
        strategy: 'fixed',
        placement: 'bottom-start',
        middleware: [offset(4), flip(), shift({ padding: 8 })],
      }).then(({ x, y }) => {
        Object.assign(this.element.style, { left: `${x}px`, top: `${y}px` })
      })
    })
  }
}

export const SlashMenu = Extension.create({
  name: 'researchObjectSlashMenu',

  addProseMirrorPlugins() {
    return [
      Suggestion<SlashMenuItem>({
        editor: this.editor,
        pluginKey: new PluginKey('researchObjectSlashMenu'),
        char: '/',
        startOfLine: true,
        allowSpaces: false,
        allow: ({ state, range }) =>
          state.doc.resolve(range.from).parent.type.name === 'paragraph',
        items: ({ editor, query }) =>
          filterSlashMenuItems(query, {
            canWrap: canWrapInResearchObject(editor),
            isWrapped: activeResearchObject(editor) !== null,
          }),
        command: ({ editor, range, props }) => {
          applySlashMenuItem(editor, range, props)
        },
        render: () => {
          let list: SlashMenuList | undefined

          return {
            onStart(props) {
              list = new SlashMenuList(props)
            },
            onUpdate(props) {
              list?.update(props)
            },
            onKeyDown({ event }) {
              if (event.key === 'Escape') {
                list?.destroy()
                list = undefined
                return true
              }
              return list?.onKeyDown(event) ?? false
            },
            onExit() {
              list?.destroy()
              list = undefined
            },
          }
        },
      }),
    ]
  },
})
