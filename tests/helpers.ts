import fs from 'fs-extra'
import path from 'path'

export const schema = (name: string) =>
  fs.readJSON(path.join(__dirname, '..', 'built', name))

export const snapshot = (name: string) =>
  path.join(__dirname, '__file_snapshots__', name)
