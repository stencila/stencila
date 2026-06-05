/**
 * Regression tests for the Tiptap synchronization client.
 *
 * These tests use small browser and editor doubles so the client can be tested
 * without constructing a real WebSocket connection or ProseMirror editor.
 */
import type { Editor } from '@tiptap/core'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { TiptapClient } from './tiptap'

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
})
