import dataProvider from '../eLifeDataProvider'

interface ArticleData {
  status: number
}

describe('eLife Data Provider ', () => {
  describe('being given a valid article id', () => {
    it('returns a 200 status code', (done: Function) => {
      const fetchMock = (): Promise<ArticleData> =>
        Promise.resolve({ status: 200 })
      return dataProvider
        .query('validArticleId', fetchMock)
        .then(data => {
          expect(data.status).toBe(200)
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
    test.todo('throws an ReferenceError')
  })
})
