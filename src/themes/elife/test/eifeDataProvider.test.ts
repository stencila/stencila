import dataProvider from '../eLifeDataProvider'

interface Response {
  ok: boolean
}

describe('eLife Data Provider ', () => {
  describe('successfully querying a valid article id', () => {
    it('does not throw', async () => {
      const fetchMock = (): Promise<Response> => Promise.resolve({ ok: true })
      await expect(
        dataProvider.query('validArticleId', fetchMock)
      ).resolves.not.toThrow()
    })

    test.todo('it exposes the URI of the article PDF')

    test.todo('it exposes the URI of the figures PDF')
  })

  describe('being given an invalid article id', () => {
    it('throws an ReferenceError', async () => {
      const fetchMock = (): Promise<Response> => Promise.resolve({ ok: false })
      await expect(
        dataProvider.query('invalidArticleId', fetchMock)
      ).rejects.toThrow(
        new ReferenceError('Invalid eLife article id: invalidArticleId')
      )
    })
  })
})
