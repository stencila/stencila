import dataProvider from '../eLifeDataProvider'

interface Response {
  ok: boolean
  json: Function
}

describe('eLife Data Provider ', () => {
  describe('successfully querying a valid article id', () => {
    it('does not throw', async () => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({ ok: true, json: () => Promise.resolve() })
      await expect(
        dataProvider.query('validArticleId', fetchMock)
      ).resolves.not.toThrow()
    })

    it('it exposes the URI of the article PDF', async () => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ pdf: 'path-to-the.pdf' })
        })
      await expect(dataProvider.query('someId', fetchMock)).resolves.toEqual({
        articleData: { pdf: 'path-to-the.pdf' },
        ok: true
      })
    })

    test.todo('it exposes the URI of the figures PDF')
  })

  describe('being given an invalid article id', () => {
    it('throws an ReferenceError', async () => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({ ok: false, json: () => Promise.resolve() })
      await expect(
        dataProvider.query('invalidArticleId', fetchMock)
      ).rejects.toThrow(
        new Error(
          `There was a problem getting article data for invalidArticleId`
        )
      )
    })
  })
})
