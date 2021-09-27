import { main } from './index'
export type ElementId = string
export type ProjectId = string
export type SnapshotId = string

declare global {
  interface Window {
    stencilaWebClient: {
      main: typeof main
    }
  }
}
