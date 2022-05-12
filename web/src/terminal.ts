import { Terminal } from 'xterm'
import { AttachAddon } from 'xterm-addon-attach'
import { FitAddon } from 'xterm-addon-fit'

import 'xterm/css/xterm.css'
import './terminal.css'

/**
 * Create an Xterm.js terminal that attaches to the Stencila document server
 *
 * @param elemId The id of the element on which to create the terminal
 */
const main = (elemId: string) => {
  const terminal = new Terminal({
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
    }/~attach`
  )
  const attachAddon = new AttachAddon(websocket)
  terminal.loadAddon(attachAddon)

  const elem = document.getElementById(elemId)
  if (!elem) {
    throw new Error(`Unable to find element with id '${elemId}'`)
  }
  terminal.open(elem)

  fitAddon.fit()
}

window.stencilaWebTerminal = {
  main,
}
