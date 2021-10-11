import { StencilaElementConstructor } from './components/base'
import { main } from './index'

export type ElementId = string
export type ProjectId = string
export type SnapshotId = string

declare global {
  interface Window {
    stencilaElements: Record<string, [StencilaElementConstructor, string]>
    stencilaWebClient: {
      main: typeof main
    }
  }
}
