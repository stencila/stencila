import fs from 'fs-extra'
import path from 'path'

export const schema = (name: string) =>
  fs.readJSON(path.join(__dirname, '..', '..', 'built', name))

export const snapshot = (dirname: string, name: string) =>
  path.join(dirname, '__file_snapshots__', name)
