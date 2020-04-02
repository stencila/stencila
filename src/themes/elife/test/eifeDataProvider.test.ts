import dataProvider from '../eLifeDataProvider'

interface Response {
  status: number
}

describe('eLife Data Provider ', () => {
  describe('being given a valid article id', () => {
    it('returns a 200 status code', (done: Function) => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({ status: 200 })
      return dataProvider
        .query('validArticleId', fetchMock)
        .then(response => {
          expect(response.status).toBe(200)
          done()
        })
        .catch((err: Error) => {
          throw err
        })
    })

    test.todo('it exposes the URI of the article PDF')

    test.todo('it exposes the URI of the figures PDF')
  })

  describe('being given an invalid article id', () => {
    it('throws an ReferenceError', async () => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({ status: 404 })
      await expect(
        dataProvider.query('invalidArticleId', fetchMock)
      ).rejects.toThrow(
        new ReferenceError('Invalid eLife article id: invalidArticleId')
      )
    })
  })
})
