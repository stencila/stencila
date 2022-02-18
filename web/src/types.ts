import { FileFormatUtils } from '@stencila/components'
import { main } from './index'

export type ElementId = string
export type ProjectId = string

declare global {
  interface Window {
    stencilaWebClient: {
      main: typeof main
      executableLanguages: FileFormatUtils.FileFormatMap
    }
  }
}
