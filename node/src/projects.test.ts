import { open, schema } from './projects'
import path from 'path'

function fixture(folder: string) {
  return path.normalize(
    path.join(__dirname, '..', '..', 'test', 'fixtures', 'projects', folder)
  )
}

describe('projects', () => {
  test('schema', () => {
    expect(schema()).toEqual(
      expect.objectContaining({
        $schema: 'http://json-schema.org/draft-07/schema#',
        title: 'Details of a project',
        type: 'object',
        properties: expect.objectContaining({
          name: expect.objectContaining({
            description: 'The name of the project',
          }),
        }),
      })
    )
  })

  test('open: empty', () => {
    let folder = fixture('empty')
    expect(open(folder)).toEqual(
      expect.objectContaining({
        path: folder,
        name: 'empty',
        theme: 'stencila',
      })
    )
  })

  test('open: manifest', () => {
    let folder = fixture('manifest')
    expect(open(folder)).toEqual(
      expect.objectContaining({
        path: folder,
        name: 'A project with a project.json file',
        theme: 'wilmore',
        mainPath: path.join(folder, 'my-main-file.md'),
        files: expect.objectContaining({
          [path.join(folder, 'project.json')]: expect.objectContaining({
            name: 'project.json',
            format: 'json',
            mediaType: 'application/json',
          }),
          [path.join(folder, 'my-main-file.md')]: expect.objectContaining({
            name: 'my-main-file.md',
            format: 'md',
            mediaType: 'text/markdown',
          }),
        }),
      })
    )
  })
})
