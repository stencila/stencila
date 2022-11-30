import { Terminal } from 'xterm'
import { AttachAddon } from 'xterm-addon-attach'
import { FitAddon } from 'xterm-addon-fit'

import 'xterm/css/xterm.css'

/**
 * Create an Xterm.js terminal that attaches to a shell session provided
 * by the Stencila document server
 *
 * Note that Xterm does not seem to like playing with the shadow root so
 * it does not seem possible to make this a true Web Component. However, for
 * consistency in code organization we put it here.
 *
 * Usage:
 *
 *   <div id="stencila-shell-terminal"><div></div></div>
 *   <script>window.stencilaShellTerminal()</script>
 */
window.stencilaShellTerminal = (directory: string) => {
  const terminal = new Terminal({
    rows: 50,
    fontFamily: 'Menlo, monospace',
    fontSize: 14,
    cursorBlink: true,
    theme: {
      foreground: '#F8F8F8',
      background: '#2D2E2C',
      selection: '#5DA5D533',
      black: '#1E1E1D',
      brightBlack: '#262625',
      red: '#CE5C5C',
      brightRed: '#FF7272',
      green: '#5BCC5B',
      brightGreen: '#72FF72',
      yellow: '#CCCC5B',
      brightYellow: '#FFFF72',
      blue: '#5D5DD3',
      brightBlue: '#7279FF',
      magenta: '#BC5ED1',
      brightMagenta: '#E572FF',
      cyan: '#5DA5D5',
      brightCyan: '#72F0FF',
      white: '#F8F8F8',
      brightWhite: '#FFFFFF',
    },
  })

  const fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)

  const websocket = new WebSocket(
    `${location.protocol === 'https:' ? 'wss' : 'ws'}://${
      location.host
    }/~shell/${directory}`
  )
  const attachAddon = new AttachAddon(websocket)
  terminal.loadAddon(attachAddon)

  const elem = document.querySelector('#stencila-shell-terminal div')!
  terminal.open(elem as HTMLElement)

  fitAddon.fit()
}
