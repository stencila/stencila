type ReceivedMessage = ModelResponseMessage

interface ModelResponseMessage {
  type: 'model-response'
  response: {
    id: string
    text: string
    files?: File[]
  }
}

type SentMessage = ChatMessage | InsertContentMessage

interface ChatCommand {
  type: 'command'
  text: string
  files?: File[]
}

interface ChatMessage extends ChatCommand {
  command: 'send-chat-message'
}

interface InsertContentMessage extends ChatCommand {
  command: 'insert-content'
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

export class ModelChatClient {
  /**
   * The render root of the view
   */
  renderRoot: HTMLElement

  constructor(renderRoot: HTMLElement) {
    this.renderRoot = renderRoot
    this.setWindowListeners()
  }

  private setWindowListeners() {
    window.addEventListener('message', (event) => this.receiveMessage(event))
    window.addEventListener(
      'stencila-model-chat-command',
      (event: CustomEvent) => {
        console.log('posting message', event.detail)
        vscode.postMessage({ type: 'command', ...event.detail })
      }
    )
  }

  receiveMessage({ data }: Event & { data: ReceivedMessage }) {
    // TODO: add receive message functionality
    console.log('message recieved', data)
  }
}
