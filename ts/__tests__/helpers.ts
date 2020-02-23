import fs from 'fs-extra'
import path from 'path'
import JsonSchema from '../jsonSchema'

export const schema = (name: string): Promise<JsonSchema> =>
  fs.readJSON(path.join(__dirname, '..', '..', 'public', name))

export const snapshot = (dirname: string, name: string): string =>
  path.join(dirname, '__file_snapshots__', name)
