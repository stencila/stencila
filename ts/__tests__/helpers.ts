import fs from 'fs-extra'
import path from 'path'
import Schema from '../schema-interface'

export const schema = (name: string): Promise<Schema> =>
  fs.readJSON(path.join(__dirname, '..', '..', 'built', name))

export const snapshot = (dirname: string, name: string): string =>
  path.join(dirname, '__file_snapshots__', name)
