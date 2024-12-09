interface ChatMessageContent {
  version: number
  /**
   * Formatted Instruciton text
   */
  text: string
  /**
   * Any files uploaded in the instruction
   */
  files: FileList
}

type ReceivedMessage = ChatMessage

type SentMessage = ChatMessage | InsertMessage

interface ChatMessage {
  type: 'instruction'
  content: ChatMessageContent
}

interface InsertMessage {
  type: 'insert-content'
  content: ChatMessageContent
}

interface VSCode {
  postMessage(message: SentMessage): void
}

/**
 * The VSCode API instance in the web view window
 *
 * Must be instantiated using `const vscode = acquireVsCodeApi()` in
 * the HTML of the view.
 */
declare const vscode: VSCode

export class ChatAssistantClient {
  /**
   * The render root of the view
   */
  renderRoot: HTMLElement

  constructor(renderRoot: HTMLElement) {
    this.renderRoot = renderRoot
  }

  receivedMessage({ data }: Event & { data: ReceivedMessage }) {
    console.log(data)
  }

  sendChatMessage(content: ChatMessageContent) {
    vscode.postMessage({
      type: 'instruction',
      content,
    })
  }

  insertContent(content: ChatMessageContent) {
    vscode.postMessage({
      type: 'insert-content',
      content,
    })
  }
}
