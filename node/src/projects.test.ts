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
      expect.arrayContaining([
        expect.stringMatching(folder),
        expect.objectContaining({
          name: 'empty',
          theme: 'stencila',
        }),
      ])
    )
  })

  test('open: manifest', () => {
    let folder = fixture('manifest')
    expect(open(folder)).toEqual(
      expect.arrayContaining([
        expect.stringMatching(folder),
        expect.objectContaining({
          name: 'A project with a project.json file',
          theme: 'wilmore',
          files: expect.objectContaining({
            [path.join(folder, 'project.json')]: expect.objectContaining({
              path: 'project.json',
              mediaType: 'application/json'
            }),
          }),
        }),
      ])
    )
  })
})
