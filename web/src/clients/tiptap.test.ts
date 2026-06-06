/**
 * Regression tests for the Tiptap synchronization client.
 *
 * These tests use small browser and editor doubles so the client can be tested
 * without constructing a real WebSocket connection or ProseMirror editor.
 */
import { Editor as TiptapEditor } from '@tiptap/core'
import type { Editor } from '@tiptap/core'
import Document from '@tiptap/extension-document'
import Paragraph from '@tiptap/extension-paragraph'
import Text from '@tiptap/extension-text'
import { undoDepth } from '@tiptap/pm/history'
import { EditorState, type Transaction } from '@tiptap/pm/state'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { createStencilaTiptapExtensions } from '../tiptap/extensions'
import { isHistoryPlugin } from '../tiptap/history'

import { TiptapClient } from './tiptap'

const EditableDocument = Document.extend({
  content: 'block*',
})

/**
 * Minimal WebSocket test double that records messages sent by the client.
 */
class FakeWebSocket {
  static instances: FakeWebSocket[] = []

  readyState = 1

  sent: string[] = []

  closed = false

  onclose?: () => void

  constructor(
    public url: string,
    public protocol: string
  ) {
    FakeWebSocket.instances.push(this)
  }

  send(message: string) {
    this.sent.push(message)
  }

  close() {
    this.readyState = 3
    this.closed = true
    this.onclose?.()
  }
}

/**
 * Install the browser globals that `Client` expects during construction.
 */
function installBrowserMocks() {
  FakeWebSocket.instances = []

  vi.stubGlobal('document', {
    body: {
      classList: {
        add: vi.fn(),
        remove: vi.fn(),
      },
    },
  })
  vi.stubGlobal('window', {
    location: {
      protocol: 'http:',
      host: 'localhost',
    },
    dispatchEvent: vi.fn(),
  })
  vi.stubGlobal('CustomEvent', class CustomEvent {
    constructor(public type: string) {}
  })
  vi.stubGlobal('WebSocket', FakeWebSocket)
}

/**
 * Create the subset of the Tiptap editor API used by `TiptapClient`.
 */
function createEditor(initialJson: unknown) {
  let json = initialJson
  let updateHandler = () => {}

  const editor = {
    on: vi.fn((_event: 'update', handler: () => void) => {
      updateHandler = handler
    }),
    off: vi.fn((_event: 'update', handler: () => void) => {
      if (handler === updateHandler) {
        updateHandler = () => {}
      }
    }),
    getJSON: vi.fn(() => json),
    commands: {
      setContent: vi.fn((value: unknown) => {
        json = value
      }),
    },
  }

  return {
    editor: editor as unknown as Editor,
    setJson(value: unknown) {
      json = value
    },
    emitUpdate() {
      updateHandler()
    },
    off: editor.off,
    setContent: editor.commands.setContent,
  }
}

function paragraphJson(text: string) {
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

function createTiptapEditor(content: unknown) {
  return new TiptapEditor({
    element: null,
    extensions: [EditableDocument, Paragraph, Text],
    content,
  })
}

function createSyncedTiptapEditor(content: unknown) {
  return new TiptapEditor({
    element: null,
    extensions: createStencilaTiptapExtensions(),
    content,
  })
}

describe('TiptapClient', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    installBrowserMocks()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('drops buffered local JSON when server content changes before debounce sends', () => {
    const initialJson = JSON.stringify({
      type: 'doc',
      content: [{ type: 'paragraph' }],
    })
    const serverJson = JSON.stringify({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [{ type: 'text', text: 'server' }],
        },
      ],
    })
    const localJson = {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [{ type: 'text', text: 'local' }],
        },
      ],
    }

    const client = new TiptapClient('doc1')
    const { editor, emitUpdate, setContent, setJson } = createEditor({
      type: 'doc',
      content: [],
    })
    client.receivePatches(editor)

    client.receiveMessage({
      version: 1,
      ops: [{ type: 'reset', insert: initialJson }],
    })

    setJson(localJson)
    emitUpdate()

    client.receiveMessage({
      version: 2,
      ops: [
        {
          type: 'replace',
          from: 0,
          to: Array.from(initialJson).length,
          insert: serverJson,
        },
      ],
    })

    vi.advanceTimersByTime(301)

    expect(setContent).toHaveBeenLastCalledWith(JSON.parse(serverJson), {
      emitUpdate: false,
    })
    expect(FakeWebSocket.instances[0].sent).toEqual([])
  })

  it('cancels buffered local JSON when editor returns to canonical content', () => {
    const initialJson = {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [{ type: 'text', text: 'initial' }],
        },
      ],
    }
    const localJson = {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [{ type: 'text', text: 'local' }],
        },
      ],
    }
    const initialString = JSON.stringify(initialJson)

    const client = new TiptapClient('doc1')
    const { editor, emitUpdate, setJson } = createEditor(initialJson)
    client.receivePatches(editor)
    client.receiveMessage({
      version: 1,
      ops: [{ type: 'reset', insert: initialString }],
    })

    setJson(localJson)
    emitUpdate()
    setJson(initialJson)
    emitUpdate()
    vi.advanceTimersByTime(301)

    expect(FakeWebSocket.instances[0].sent).toEqual([])
  })

  it('unregisters editor updates and closes the socket when destroyed', () => {
    const client = new TiptapClient('doc1')
    const { editor, emitUpdate, off, setJson } = createEditor({
      type: 'doc',
      content: [],
    })

    client.receivePatches(editor)
    client.destroy()

    setJson({
      type: 'doc',
      content: [{ type: 'paragraph' }],
    })
    emitUpdate()
    vi.advanceTimersByTime(301)

    expect(off).toHaveBeenCalled()
    expect(FakeWebSocket.instances[0].closed).toBe(true)
    expect(FakeWebSocket.instances[0].sent).toEqual([])
  })

  it('does not replace editor content when server JSON already matches', () => {
    const json = paragraphJson('hello')
    const serverJson = {
      content: json.content,
      type: 'doc',
    }
    const client = new TiptapClient('doc1')
    const { editor, setContent } = createEditor(json)

    client.receivePatches(editor)
    client.receiveMessage({
      version: 1,
      ops: [{ type: 'reset', insert: JSON.stringify(serverJson, null, 2) }],
    })

    expect(setContent).not.toHaveBeenCalled()
  })

  it('does not replace editor content when server JSON is semantically equivalent', () => {
    const serverJson = JSON.stringify({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [],
        },
      ],
    })

    vi.unstubAllGlobals()
    const editor = createSyncedTiptapEditor({
      type: 'doc',
      content: [{ type: 'paragraph' }],
    })
    installBrowserMocks()
    const client = new TiptapClient('doc1')
    let docChanged = false

    try {
      editor.on('transaction', ({ transaction }) => {
        docChanged ||= transaction.docChanged
      })

      client.receivePatches(editor)
      client.receiveMessage({
        version: 1,
        ops: [{ type: 'reset', insert: serverJson }],
      })

      expect(editor.getJSON()).toEqual({
        type: 'doc',
        content: [{ type: 'paragraph' }],
      })
      expect(docChanged).toBe(false)
    } finally {
      client.destroy()
      editor.destroy()
    }
  })

  it('keeps a pending undo when the server echoes the previous edit', () => {
    const initialJson = paragraphJson('hello')
    const editedJson = paragraphJson('hello world')
    const initialString = JSON.stringify(initialJson)
    const prettyEditedString = JSON.stringify(editedJson, null, 2)

    vi.unstubAllGlobals()
    const editor = createSyncedTiptapEditor(initialJson)
    installBrowserMocks()
    const client = new TiptapClient('doc1')

    try {
      client.receivePatches(editor)
      client.receiveMessage({
        version: 1,
        ops: [{ type: 'reset', insert: initialString }],
      })

      editor.commands.setTextSelection(6)
      editor.commands.insertContent({ type: 'text', text: ' world' })
      vi.advanceTimersByTime(301)

      expect(editor.getText()).toBe('hello world')
      expect(FakeWebSocket.instances[0].sent).toHaveLength(1)

      expect(editor.commands.setContent(initialJson, { emitUpdate: true })).toBe(
        true
      )
      expect(editor.getText()).toBe('hello')

      client.receiveMessage({
        version: 2,
        ops: [{ type: 'reset', insert: prettyEditedString }],
      })

      expect(editor.getText()).toBe('hello')

      vi.advanceTimersByTime(301)

      expect(FakeWebSocket.instances[0].sent).toHaveLength(2)
      expect(
        JSON.parse(
          JSON.parse(FakeWebSocket.instances[0].sent[1]).ops[0].insert
        )
      ).toEqual(initialJson)
    } finally {
      client.destroy()
      editor.destroy()
    }
  })

  it('clears stale undo history when a server reset discards local content', () => {
    const initialJson = paragraphJson('hello')
    const serverJson = paragraphJson('server')
    const initialString = JSON.stringify(initialJson)
    const serverString = JSON.stringify(serverJson)

    vi.unstubAllGlobals()
    const tiptapEditor = createSyncedTiptapEditor(initialJson)
    installBrowserMocks()
    const client = new TiptapClient('doc1')
    const historyPlugin =
      tiptapEditor.extensionManager.plugins.find(isHistoryPlugin)
    if (!historyPlugin) {
      throw new Error('Expected history plugin')
    }

    let updateHandler = () => {}
    let state = EditorState.create({
      schema: tiptapEditor.schema,
      doc: tiptapEditor.schema.nodeFromJSON(initialJson),
      plugins: [historyPlugin],
    })
    const editor = {
      schema: tiptapEditor.schema,
      options: tiptapEditor.options,
      get state() {
        return state
      },
      view: {
        dispatch(transaction: Transaction) {
          state = state.apply(transaction)
        },
        updateState(nextState: EditorState) {
          state = nextState
        },
      },
      on: vi.fn((_event: 'update', handler: () => void) => {
        updateHandler = handler
      }),
      off: vi.fn(),
      getJSON: vi.fn(() => state.doc.toJSON()),
      commands: {
        setContent: vi.fn(),
      },
    } as unknown as Editor

    try {
      client.receivePatches(editor)
      client.receiveMessage({
        version: 1,
        ops: [{ type: 'reset', insert: initialString }],
      })

      state = state.apply(state.tr.insertText(' world', 6))
      updateHandler()

      expect(state.doc.textContent).toBe('hello world')
      expect(undoDepth(state)).toBe(1)

      client.receiveMessage({
        version: 2,
        ops: [{ type: 'reset', insert: serverString }],
      })

      expect(state.doc.textContent).toBe('server')
      expect(undoDepth(state)).toBe(0)
    } finally {
      client.destroy()
      tiptapEditor.destroy()
    }
  })

  it('preserves editor selection when replacing content from the server', () => {
    const json = paragraphJson('hello world')
    const compactJson = JSON.stringify(json)
    const prettyJson = JSON.stringify(json, null, 2)

    vi.unstubAllGlobals()
    const editor = createTiptapEditor(json)
    installBrowserMocks()
    const client = new TiptapClient('doc1')

    try {
      client.receivePatches(editor)
      client.receiveMessage({
        version: 1,
        ops: [{ type: 'reset', insert: compactJson }],
      })

      editor.commands.setTextSelection({ from: 7, to: 2 })

      client.receiveMessage({
        version: 2,
        ops: [{ type: 'reset', insert: prettyJson }],
      })

      expect(editor.state.selection.from).toBe(2)
      expect(editor.state.selection.to).toBe(7)
      expect(editor.state.selection.anchor).toBe(7)
      expect(editor.state.selection.head).toBe(2)
    } finally {
      client.destroy()
      editor.destroy()
    }
  })

  it('collapses initial empty-editor selection after server content loads', () => {
    const json = paragraphJson('hello world')
    const compactJson = JSON.stringify(json)

    vi.unstubAllGlobals()
    const editor = createTiptapEditor({
      type: 'doc',
      content: [],
    })
    installBrowserMocks()
    const client = new TiptapClient('doc1')

    try {
      client.receivePatches(editor)
      client.receiveMessage({
        version: 1,
        ops: [{ type: 'reset', insert: compactJson }],
      })

      expect(editor.state.selection.empty).toBe(true)
      expect(editor.state.selection.anchor).toBe(1)
      expect(editor.state.selection.head).toBe(1)
    } finally {
      client.destroy()
      editor.destroy()
    }
  })

  it('does not make server content replacements undoable', () => {
    const json = paragraphJson('hello')
    const compactJson = JSON.stringify(json)

    vi.unstubAllGlobals()
    const editor = createSyncedTiptapEditor({
      type: 'doc',
      content: [],
    })
    installBrowserMocks()
    const client = new TiptapClient('doc1')
    let addToHistoryMeta: unknown

    try {
      editor.on('transaction', ({ transaction }) => {
        if (transaction.docChanged) {
          addToHistoryMeta = transaction.getMeta('addToHistory')
        }
      })
      client.receivePatches(editor)
      client.receiveMessage({
        version: 1,
        ops: [{ type: 'reset', insert: compactJson }],
      })

      expect(editor.getText()).toBe('hello')
      expect(addToHistoryMeta).toBe(false)
      expect(editor.can().undo()).toBe(false)
    } finally {
      client.destroy()
      editor.destroy()
    }
  })
})
