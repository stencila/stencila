import { FileFormatUtils } from '@stencila/components'
import { main } from './index'

export type ElementId = string

declare global {
  interface Window {
    stencilaWebClient: {
      main: typeof main
      executableLanguages: FileFormatUtils.FileFormatMap
    }
    stencilaWebTerminal: {
      main: (elemId: string) => void
    }
  }
}
