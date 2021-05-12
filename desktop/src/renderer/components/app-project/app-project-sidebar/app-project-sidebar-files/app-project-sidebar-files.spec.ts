import { newSpecPage } from '@stencil/core/testing'
import { AppProjectSidebarFiles } from './app-project-sidebar-files'

describe('app-project-sidebar-files', () => {
  describe.skip('normalization', () => {
    it('returns a blank string if the name is undefined', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppProjectSidebarFiles],
        html: '<app-project-sidebar-files></app-project-sidebar-files>',
      })
      expect(rootInstance.normalize(undefined)).toEqual('')
    })

    it('returns a blank string if the name is null', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppProjectSidebarFiles],
        html: '<app-project-sidebar-files></app-project-sidebar-files>',
      })
      expect(rootInstance.normalize(null)).toEqual('')
    })

    it('capitalizes the first letter', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppProjectSidebarFiles],
        html: '<app-project-sidebar-files></app-project-sidebar-files>',
      })
      expect(rootInstance.normalize('quincy')).toEqual('Quincy')
    })

    it('lower-cases the following letters', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppProjectSidebarFiles],
        html: '<app-project-sidebar-files></app-project-sidebar-files>',
      })
      expect(rootInstance.normalize('JOSEPH')).toEqual('Joseph')
    })

    it('handles single letter names', async () => {
      const { rootInstance } = await newSpecPage({
        components: [AppProjectSidebarFiles],
        html: '<app-project-sidebar-files></app-project-sidebar-files>',
      })
      expect(rootInstance.normalize('q')).toEqual('Q')
    })
  })
})
