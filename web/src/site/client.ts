import { reload, reloadStyles } from './glide'

/**
 * Initialize the site client for live reload in preview mode
 *
 * Only creates the client when running on localhost (preview mode)
 */
export function initSiteClient(): SiteClient | null {
  const host = window.location.hostname
  if (host === 'localhost' || host === '127.0.0.1') {
    return new SiteClient()
  }
  return null
}

/**
 * Message types from the site WebSocket server
 */
interface SiteMessage {
  type: 'ConfigChange' | 'SiteChange' | 'ThemeChange' | 'Error'
  paths?: string[]
  message?: string
}

/**
 * WebSocket client for site preview live reload
 *
 * Connects to the server's site watching endpoint and triggers
 * smooth content updates using the glide navigation module instead
 * of full page reloads, avoiding flash of unstyled content.
 */
class SiteClient {
  /**
   * The WebSocket connection
   */
  private ws: WebSocket | null = null

  /**
   * Whether the client has connected at least once
   */
  private hasConnected: boolean = false

  /**
   * Current reconnection attempt count
   */
  private reconnectAttempts: number = 0

  /**
   * Maximum reconnection attempts
   */
  private maxReconnectAttempts: number = 10

  /**
   * Reconnection interval in milliseconds
   */
  private reconnectInterval: number = 1000

  /**
   * Whether the client should attempt to reconnect
   */
  private shouldReconnect: boolean = true

  /**
   * Create a new site client and connect
   */
  constructor() {
    this.connect()
  }

  /**
   * Connect to the site WebSocket
   */
  private connect() {
    const protocol = window.location.protocol === 'http:' ? 'ws' : 'wss'
    const url = `${protocol}://${window.location.host}/~site/websocket`

    this.ws = new WebSocket(url)

    this.ws.onopen = () => {
      console.debug('🔌 SiteClient connected')

      this.hasConnected = true
      this.reconnectAttempts = 0
    }

    this.ws.onmessage = (event) => {
      try {
        const message: SiteMessage = JSON.parse(event.data)

        switch (message.type) {
          case 'ConfigChange':
          case 'SiteChange':
            console.debug(`🔄 SiteClient ${message.type}:`, message.paths ?? '')
            void reload()
            break
          case 'ThemeChange':
            console.debug('🎨 SiteClient theme changed, reloading styles...')
            reloadStyles()
            break
          case 'Error':
            console.error('❌ SiteClient error:', message.message)
            break
        }
      } catch (e) {
        console.error('❌ SiteClient failed to parse message:', e)
      }
    }

    this.ws.onclose = () => {
      if (this.hasConnected) {
        console.debug('🔌 SiteClient disconnected')
      }

      if (this.shouldReconnect && this.reconnectAttempts < this.maxReconnectAttempts) {
        this.reconnectAttempts++
        console.debug(
          `🔌 SiteClient reconnecting (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`
        )
        setTimeout(() => this.connect(), this.reconnectInterval)
      }
    }

    this.ws.onerror = () => {
      // Silently ignore errors - this is expected in production
      // where the endpoint doesn't exist
    }
  }

  /**
   * Disconnect from the site WebSocket
   */
  disconnect() {
    this.shouldReconnect = false
    this.ws?.close()
  }
}
