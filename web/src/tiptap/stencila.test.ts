/**
 * Unit tests for Stencila-specific Tiptap extensions and their JSON shape.
 */
import {
  Editor,
  type InputRule,
  type InputRuleFinder,
  type JSONContent,
  createChainableState,
  getSchema,
} from '@tiptap/core'
import { redoDepth, undoDepth } from '@tiptap/pm/history'
import { EditorState, TextSelection, type Transaction } from '@tiptap/pm/state'
import type { EditorView } from '@tiptap/pm/view'
import { describe, expect, it } from 'vitest'

import { createStencilaTiptapExtensions } from './extensions'
import { isHistoryPlugin } from './history'
import { attrsWithUpdatedMathSource } from './math'

/**
 * Create a Tiptap editor using the same core extensions as the edit view.
 */
function createEditor(content: JSONContent): Editor {
  return new Editor({
    element: null,
    extensions: createStencilaTiptapExtensions(),
    content,
  })
}

function paragraphJson(text: string): JSONContent {
  return {
    type: 'doc',
    content: [
      {
        type: 'paragraph',
        content: [{ type: 'text', text }],
      },
    ],
  }
}

function markedParagraphJson(
  text: string,
  marks: JSONContent['marks']
): JSONContent {
  return {
    type: 'doc',
    content: [
      {
        type: 'paragraph',
        content: [
          {
            type: 'text',
            marks,
            text,
          },
        ],
      },
    ],
  }
}

function stencilaParagraphJson(text: string): Record<string, unknown> {
  return {
    type: 'Paragraph',
    content: [
      {
        type: 'Text',
        value: {
          string: text,
        },
      },
    ],
  }
}

type InputRuleMatch = RegExpMatchArray & {
  data?: Record<string, unknown>
}

function matchInputRule(
  find: InputRuleFinder,
  text: string
): InputRuleMatch | null {
  if (find instanceof RegExp) {
    return find.exec(text) as InputRuleMatch | null
  }

  const result = find(text)

  if (!result) {
    return null
  }

  const match = [result.text] as unknown as InputRuleMatch

  match.index = result.index
  match.input = text
  match.data = result.data

  if (result.replaceWith) {
    match.push(result.replaceWith)
  }

  return match
}

function inputRulesFor(editor: Editor, extensionName: string): InputRule[] {
  const extension = editor.extensionManager.extensions.find(
    ({ name }) => name === extensionName
  )

  return (
    extension?.config.addInputRules?.call({
      editor,
      extensions: [],
      name: extension.name,
      options: extension.options,
      parent: undefined,
      storage: {},
      type:
        editor.schema.marks[extensionName] ?? editor.schema.nodes[extensionName],
    } as never) ?? []
  )
}

function applyInputRuleShortcut(
  shortcut: string,
  extensionName: string
): JSONContent {
  const finalText = shortcut.slice(-1)

  if (!finalText) {
    return paragraphJson(shortcut)
  }

  const initialText = shortcut.slice(0, -1)
  const editor = createEditor(paragraphJson(initialText))

  try {
    const ruleMatch = inputRulesFor(editor, extensionName)
      .map((rule) => ({ match: matchInputRule(rule.find, shortcut), rule }))
      .find(({ match }) => match)

    if (!ruleMatch?.match) {
      return paragraphJson(shortcut)
    }

    let state = EditorState.create({
      schema: editor.schema,
      doc: editor.schema.nodeFromJSON(paragraphJson(initialText)),
    })
    const cursor = state.doc.content.size - 1

    state = state.apply(
      state.tr.setSelection(TextSelection.create(state.doc, cursor))
    )

    const transaction = state.tr
    const range = {
      from: state.selection.from - (ruleMatch.match[0].length - finalText.length),
      to: state.selection.to,
    }

    ruleMatch.rule.handler({
      can: (() => ({})) as never,
      chain: (() => ({})) as never,
      commands: {} as never,
      match: ruleMatch.match,
      range,
      state: createChainableState({ state, transaction }),
    })

    if (transaction.steps.length === 0) {
      return paragraphJson(shortcut)
    }

    return state.apply(transaction).doc.toJSON() as JSONContent
  } finally {
    editor.destroy()
  }
}

/**
 * Supported native marks plus opaque Stencila extension nodes.
 */
function supportedTiptapJson(): JSONContent {
  return {
    type: 'doc',
    content: [
      {
        type: 'heading',
        attrs: { level: 2 },
        content: [{ type: 'text', text: 'Title' }],
      },
      {
        type: 'paragraph',
        content: [
          { type: 'text', text: 'Hello ' },
          {
            type: 'text',
            marks: [{ type: 'bold' }],
            text: 'bold',
          },
          { type: 'text', text: ' and ' },
          {
            type: 'text',
            marks: [{ type: 'italic' }],
            text: 'italic',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [
              {
                type: 'link',
                attrs: {
                  href: 'https://example.com',
                  title: 'Example',
                  rel: 'noopener',
                  labelOnly: true,
                },
              },
            ],
            text: 'link',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [
              {
                type: 'code',
                attrs: {
                  programmingLanguage: 'typescript',
                },
              },
            ],
            text: 'code',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'strike' }],
            text: 'strike',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'underline' }],
            text: 'under',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'subscript' }],
            text: 'sub',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'superscript' }],
            text: 'sup',
          },
          { type: 'text', text: ' ' },
          {
            type: 'mathInline',
            attrs: {
              id: null,
              code: 'x + y',
              mathLanguage: 'tex',
              compilationMessages: null,
              mathml: null,
              images: null,
            },
          },
        ],
      },
      {
        type: 'blockquote',
        content: [
          {
            type: 'paragraph',
            content: [{ type: 'text', text: 'Quoted' }],
          },
        ],
      },
      {
        type: 'bulletList',
        content: [
          {
            type: 'listItem',
            content: [
              {
                type: 'paragraph',
                content: [{ type: 'text', text: 'Bullet' }],
              },
            ],
          },
        ],
      },
      {
        type: 'taskList',
        content: [
          {
            type: 'taskItem',
            attrs: { checked: true },
            content: [
              {
                type: 'paragraph',
                content: [{ type: 'text', text: 'Done' }],
              },
            ],
          },
          {
            type: 'taskItem',
            attrs: { checked: false },
            content: [
              {
                type: 'paragraph',
                content: [{ type: 'text', text: 'Todo' }],
              },
            ],
          },
        ],
      },
      {
        type: 'orderedList',
        attrs: { start: 3, type: null },
        content: [
          {
            type: 'listItem',
            content: [
              {
                type: 'paragraph',
                content: [{ type: 'text', text: 'Third' }],
              },
            ],
          },
          {
            type: 'listItem',
            content: [
              {
                type: 'paragraph',
                content: [{ type: 'text', text: 'Fourth' }],
              },
            ],
          },
        ],
      },
      {
        type: 'codeBlock',
        attrs: {
          id: 'code-1',
          isDemo: true,
          language: 'typescript',
        },
        content: [{ type: 'text', text: 'const value = 1' }],
      },
      {
        type: 'mathBlock',
        attrs: {
          id: 'eq-1',
          code: 'x + y',
          mathLanguage: 'tex',
          compilationMessages: null,
          mathml: null,
          images: null,
          idAutomatically: null,
          label: '1',
          labelAutomatically: true,
        },
      },
      {
        type: 'horizontalRule',
      },
      {
        type: 'table',
        attrs: {
          id: 'table-1',
          label: 'Table 1',
          labelAutomatically: true,
          caption: [stencilaParagraphJson('Caption')],
          notes: [stencilaParagraphJson('Note')],
        },
        content: [
          {
            type: 'tableRow',
            content: [
              {
                type: 'tableHeader',
                attrs: {
                  align: null,
                  colspan: 1,
                  rowspan: 1,
                  colwidth: null,
                },
                content: [
                  {
                    type: 'paragraph',
                    content: [{ type: 'text', text: 'Head' }],
                  },
                ],
              },
            ],
          },
          {
            type: 'tableRow',
            content: [
              {
                type: 'tableCell',
                attrs: {
                  align: null,
                  colspan: 1,
                  rowspan: 1,
                  colwidth: null,
                },
                content: [
                  {
                    type: 'paragraph',
                    content: [{ type: 'text', text: 'Data' }],
                  },
                ],
              },
            ],
          },
        ],
      },
      {
        type: 'stencilaBlock',
        attrs: {
          nodeType: 'CodeChunk',
          node: { type: 'CodeChunk', code: "print('hello')" },
        },
      },
    ],
  }
}

describe('Stencila Tiptap extensions', () => {
  it('serializes supported and opaque nodes in canonical JSON shape', () => {
    const editor = createEditor(supportedTiptapJson())

    try {
      expect(editor.getJSON()).toEqual(supportedTiptapJson())
    } finally {
      editor.destroy()
    }
  })

  it('registers math nodes', () => {
    const schema = getSchema(createStencilaTiptapExtensions())

    expect(schema.nodes.mathBlock).toBeDefined()
    expect(schema.nodes.mathInline).toBeDefined()
  })

  it('creates inline math from dollar shortcuts', () => {
    expect(applyInputRuleShortcut('$x + y$', 'mathInline')).toMatchObject({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            {
              type: 'mathInline',
              attrs: {
                code: 'x + y',
                mathLanguage: 'tex',
              },
            },
          ],
        },
      ],
    })
  })

  it('creates block math from double-dollar shortcuts', () => {
    expect(applyInputRuleShortcut('$$x + y$$', 'mathBlock')).toMatchObject({
      type: 'doc',
      content: [
        {
          type: 'mathBlock',
          attrs: {
            code: 'x + y',
            mathLanguage: 'tex',
          },
        },
      ],
    })
  })

  it('does not create inline math inside double-dollar shortcuts', () => {
    expect(applyInputRuleShortcut('$$x + y$$', 'mathInline')).toEqual(
      paragraphJson('$$x + y$$')
    )
  })

  it('clears stale math output after editing source attrs', () => {
    expect(
      attrsWithUpdatedMathSource(
        {
          id: 'eq-1',
          code: 'x',
          mathLanguage: 'tex',
          compilationMessages: [{ level: 'Warning' }],
          mathml: '<math><mi>x</mi></math>',
          images: [{ contentUrl: 'old.png' }],
        },
        'x + y'
      )
    ).toEqual({
      id: 'eq-1',
      code: 'x + y',
      mathLanguage: 'tex',
      compilationMessages: null,
      mathml: null,
      images: null,
    })
  })

  it('preserves code block attrs after editing code text', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          attrs: {
            id: 'code-1',
            isDemo: true,
            language: 'typescript',
          },
          content: [{ type: 'text', text: 'const value = 1' }],
        },
      ],
    })

    try {
      const insertAt = editor.state.doc.content.size - 1

      editor.view.dispatch(
        editor.state.tr.insertText('\nconst next = value + 1', insertAt)
      )

      const [codeBlock] = editor.getJSON().content ?? []

      expect(codeBlock).toMatchObject({
        type: 'codeBlock',
        attrs: {
          id: 'code-1',
          isDemo: true,
          language: 'typescript',
        },
      })
    } finally {
      editor.destroy()
    }
  })

  it('preserves table attrs after editing cell text', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'table',
          attrs: {
            id: 'table-1',
            label: 'Table 1',
            labelAutomatically: true,
            caption: [stencilaParagraphJson('Caption')],
            notes: [stencilaParagraphJson('Note')],
          },
          content: [
            {
              type: 'tableRow',
              content: [
                {
                  type: 'tableCell',
                  attrs: {
                    align: null,
                    colspan: 1,
                    rowspan: 1,
                    colwidth: null,
                  },
                  content: [
                    {
                      type: 'paragraph',
                      content: [{ type: 'text', text: 'Data' }],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ],
    })

    try {
      let insertAt: number | undefined

      editor.state.doc.descendants((node, position) => {
        if (node.isText && node.text === 'Data') {
          insertAt = position + node.nodeSize
          return false
        }

        return true
      })

      if (insertAt === undefined) {
        throw new Error('Expected table cell text position')
      }

      editor.view.dispatch(editor.state.tr.insertText(' edited', insertAt))

      const [table] = editor.getJSON().content ?? []

      expect(table).toMatchObject({
        type: 'table',
        attrs: {
          id: 'table-1',
          label: 'Table 1',
          labelAutomatically: true,
          caption: [stencilaParagraphJson('Caption')],
          notes: [stencilaParagraphJson('Note')],
        },
      })
    } finally {
      editor.destroy()
    }
  })

  it('registers task list nodes', () => {
    const schema = getSchema(createStencilaTiptapExtensions())

    expect(schema.nodes.taskList).toBeDefined()
    expect(schema.nodes.taskItem).toBeDefined()
  })

  it('defines link marks as anchor elements', () => {
    const schema = getSchema(createStencilaTiptapExtensions())
    const render = schema.marks.link.spec.toDOM
    const mark = schema.marks.link.create({
      href: 'https://example.com',
      title: 'Example',
      rel: 'noopener',
      labelOnly: true,
    })

    expect(render?.(mark, true)).toEqual([
      'a',
      {
        href: 'https://example.com',
        title: 'Example',
        rel: 'noopener',
      },
      0,
    ])
  })

  it('converts inline Markdown shortcuts to custom marks', () => {
    expect(applyInputRuleShortcut('`code`', 'code')).toEqual(
      markedParagraphJson('code', [
        { type: 'code', attrs: { programmingLanguage: null } },
      ])
    )
    expect(applyInputRuleShortcut('~~strike~~', 'strike')).toEqual(
      markedParagraphJson('strike', [{ type: 'strike' }])
    )
    expect(applyInputRuleShortcut('H~2~', 'subscript')).toEqual({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'H' },
            {
              type: 'text',
              marks: [{ type: 'subscript' }],
              text: '2',
            },
          ],
        },
      ],
    })
    expect(applyInputRuleShortcut('CO^2^', 'superscript')).toEqual({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'CO' },
            {
              type: 'text',
              marks: [{ type: 'superscript' }],
              text: '2',
            },
          ],
        },
      ],
    })
    expect(
      applyInputRuleShortcut('[Stencila](https://stencila.io)', 'link')
    ).toEqual(
      markedParagraphJson('Stencila', [
        {
          type: 'link',
          attrs: {
            href: 'https://stencila.io',
            labelOnly: null,
            rel: null,
            title: null,
          },
        },
      ])
    )
    expect(
      applyInputRuleShortcut('[Stencila](https://stencila.io "Home")', 'link')
    ).toEqual(
      markedParagraphJson('Stencila', [
        {
          type: 'link',
          attrs: {
            href: 'https://stencila.io',
            labelOnly: null,
            rel: null,
            title: 'Home',
          },
        },
      ])
    )
  })

  it('keeps unsupported inline Markdown shortcuts as text', () => {
    expect(applyInputRuleShortcut('[Stencila]()', 'link')).toEqual(
      paragraphJson('[Stencila]()')
    )
    expect(applyInputRuleShortcut('H~2 3~', 'subscript')).toEqual(
      paragraphJson('H~2 3~')
    )
    expect(applyInputRuleShortcut('~~strike~', 'subscript')).toEqual(
      paragraphJson('~~strike~')
    )
  })

  it('registers the link click handler', () => {
    const editor = createEditor({
      type: 'doc',
      content: [],
    })

    try {
      expect(
        editor.extensionManager.plugins.some((plugin) =>
          Boolean(plugin.props.handleClick)
        )
      ).toBe(true)
    } finally {
      editor.destroy()
    }
  })

  it('registers history commands backed by the history plugin', () => {
    const editor = createEditor(paragraphJson('hello'))
    const historyPlugin =
      editor.extensionManager.plugins.find(isHistoryPlugin)
    const undo = editor.extensionManager.commands.undo
    const redo = editor.extensionManager.commands.redo

    expect(historyPlugin).toBeDefined()
    expect(undo).toBeTypeOf('function')
    expect(redo).toBeTypeOf('function')

    try {
      let state = EditorState.create({
        schema: editor.schema,
        doc: editor.schema.nodeFromJSON(paragraphJson('hello')),
        plugins: historyPlugin ? [historyPlugin] : [],
      })
      const view = {
        dispatch(transaction: Transaction) {
          state = state.apply(transaction)
        },
      } as EditorView
      const commandProps = () =>
        ({
          editor: {
            get state() {
              return state
            },
          },
          dispatch: (): undefined => undefined,
          state,
          tr: state.tr,
          view,
        }) as never

      state = state.apply(state.tr.insertText(' world', 6))

      expect(state.doc.textContent).toBe('hello world')
      expect(undoDepth(state)).toBe(1)

      expect(undo()(commandProps())).toBe(true)
      expect(state.doc.textContent).toBe('hello')
      expect(redoDepth(state)).toBe(1)

      expect(redo()(commandProps())).toBe(true)
      expect(state.doc.textContent).toBe('hello world')
    } finally {
      editor.destroy()
    }
  })

  it('registers undo and redo keyboard shortcuts', () => {
    const editor = createEditor(paragraphJson('hello'))
    const historyExtension = editor.extensionManager.extensions.find(
      (extension) => extension.name === 'history'
    )
    const shortcuts = historyExtension?.config.addKeyboardShortcuts?.call({
      editor,
      extensions: [],
      name: 'history',
      options: historyExtension.options,
      parent: undefined,
      storage: {},
      type: undefined,
    })

    try {
      expect(Object.keys(shortcuts ?? {}).sort()).toEqual([
        'Mod-Shift-z',
        'Mod-y',
        'Mod-z',
      ])
    } finally {
      editor.destroy()
    }
  })
})
