import { newSpecPage } from '@stencil/core/testing'
import { AppDocumentEditor } from './app-document-editor'

describe('app-project-file-preview', () => {
  describe.skip('normalization', () => {
    it('returns a blank string if the name is undefined', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppDocumentEditor],
        html: '<app-project-file-preview></app-project-file-preview>',
      })
      expect(rootInstance.normalize(undefined)).toEqual('')
    })

    it('returns a blank string if the name is null', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppDocumentEditor],
        html: '<app-project-file-preview></app-project-file-preview>',
      })
      expect(rootInstance.normalize(null)).toEqual('')
    })

    it('capitalizes the first letter', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppDocumentEditor],
        html: '<app-project-file-preview></app-project-file-preview>',
      })
      expect(rootInstance.normalize('quincy')).toEqual('Quincy')
    })

    it('lower-cases the following letters', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppDocumentEditor],
        html: '<app-project-file-preview></app-project-file-preview>',
      })
      expect(rootInstance.normalize('JOSEPH')).toEqual('Joseph')
    })

    it('handles single letter names', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppDocumentEditor],
        html: '<app-project-file-preview></app-project-file-preview>',
      })
      expect(rootInstance.normalize('q')).toEqual('Q')
    })
  })
})
