import { ThemeManager } from '../ui/document/themes'

/**
 * Message types from the theme WebSocket server
 */
interface ThemeMessage {
  type: 'ThemeUpdate' | 'Error'
  theme_type?: string
  content?: string
  name?: string
  message?: string
}

/**
 * WebSocket client for live theme reloading
 *
 * Connects to the server's theme watching endpoint and receives
 * updates when workspace or user theme CSS files change.
 */
export class ThemeClient {
  /**
   * The WebSocket connection
   */
  private ws: WebSocket | null = null

  /**
   * Whether the client has connected at least once
   */
  private hasConnected: boolean = false

  /**
   * Initial reconnection interval in milliseconds
   */
  private initialReconnectInterval: number = 1000

  /**
   * Current reconnection interval
   */
  private currentReconnectInterval: number = this.initialReconnectInterval

  /**
   * Whether the client should attempt to reconnect
   */
  private shouldReconnect: boolean = true

  /**
   * Create a new theme client
   *
   * @param themeType The type of theme to watch ('workspace' or 'user')
   * @param themeName The name of the theme (required for user themes)
   */
  constructor(
    private themeType: 'workspace' | 'user',
    private themeName?: string
  ) {
    this.connect()
  }

  /**
   * Connect to the theme WebSocket
   */
  private connect() {
    const protocol = window.location.protocol === 'http:' ? 'ws' : 'wss'
    let url = `${protocol}://${window.location.host}/~themes/websocket?theme-type=${this.themeType}`

    // Add theme name for user themes
    if (this.themeType === 'user' && this.themeName) {
      url += `&theme-name=${encodeURIComponent(this.themeName)}`
    }

    this.ws = new WebSocket(url)

    this.ws.onopen = () => {
      console.debug('🔌 ThemeClient connected')

      const classList = document.body.classList
      classList.add('theme-client-connected')
      classList.remove('theme-client-disconnected')

      window.dispatchEvent(
        new CustomEvent(
          this.hasConnected ? 'theme-client-reconnected' : 'theme-client-connected'
        )
      )

      this.hasConnected = true
      this.currentReconnectInterval = this.initialReconnectInterval
    }

    this.ws.onmessage = (event) => {
      try {
        const message: ThemeMessage = JSON.parse(event.data)

        if (message.type === 'ThemeUpdate' && message.content) {
          console.debug('🎨 ThemeClient update received')
          ThemeManager.updateThemeCSS(message.content)
        } else if (message.type === 'Error') {
          console.error('❌ ThemeClient update error:', message.message)
        }
      } catch (e) {
        console.error('❌ ThemeClient failed to parse theme message:', e)
      }
    }

    this.ws.onclose = () => {
      console.debug('🔌 ThemeClient disconnected')

      document.body.classList.add('theme-client-disconnected')

      window.dispatchEvent(new CustomEvent('theme-client-disconnected'))

      if (this.shouldReconnect) {
        setTimeout(
          () => {
            if (this.currentReconnectInterval < 120000) {
              this.currentReconnectInterval *= 1.5
            }
            this.connect()
          },
          this.currentReconnectInterval + Math.random() * 3000
        )
      }
    }

    this.ws.onerror = (error) => {
      console.error('❌ ThemeClient WebSocket error:', error)
    }
  }

  /**
   * Disconnect from the theme WebSocket
   *
   * Call this when the client should no longer watch for theme changes
   */
  disconnect() {
    this.shouldReconnect = false
    this.ws?.close()
  }
}
