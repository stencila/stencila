import { open, schema } from './projects'
import path from 'path'

function fixture(folder: string) {
  return path.join(
    __dirname,
    '..',
    '..',
    'test',
    'fixtures',
    'projects',
    folder
  )
}

describe('projects', () => {
  test('schema', () => {
    expect(schema()).toEqual(
      expect.objectContaining({
        $schema: 'http://json-schema.org/draft-07/schema#',
        title: 'Description of a project',
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
    let path = fixture('empty')
    expect(open(path)).toEqual(
      expect.arrayContaining([
        expect.stringMatching(/empty$/),
        expect.objectContaining({
          name: 'empty',
          theme: "stencila"
        }),
      ])
    )
  })

  test('open: manifest', () => {
    let path = fixture('manifest')
    expect(open(path)).toEqual(
      expect.arrayContaining([
        expect.stringMatching(/manifest$/),
        expect.objectContaining({
          name: 'A project with a project.json file',
          theme: 'wilmore'
        }),
      ])
    )
  })
})
