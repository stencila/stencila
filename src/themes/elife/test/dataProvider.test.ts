import * as dataProvider from '../lib/dataProvider'

interface Response {
  ok: boolean
  json: Function
}
const body = document.body

const resetDom = (): void => {
  body.innerHTML = ''
}

describe('data Provider ', () => {
  afterEach(resetDom)

  describe('query', () => {
    describe('successfully querying a valid article id', () => {
      it('does not throw', async () => {
        const fetchMock = (): Promise<Response> =>
          Promise.resolve({ ok: true, json: () => Promise.resolve() })
        await expect(
          dataProvider.query('validArticleId', fetchMock)
        ).resolves.not.toThrow()
      })

      it('it exposes the url of the article PDF', async () => {
        const fetchMock = (): Promise<Response> =>
          Promise.resolve({
            ok: true,
            json: () => Promise.resolve({ pdf: 'path-to-the.pdf' }),
          })
        await expect(dataProvider.query('someId', fetchMock)).resolves.toEqual({
          articleData: { pdf: 'path-to-the.pdf' },
          ok: true,
        })
      })

      it('it exposes the url of the figures PDF', async () => {
        const fetchMock = (): Promise<Response> =>
          Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                figuresPdf: 'path-to-the-figures.pdf',
              }),
          })
        await expect(dataProvider.query('someId', fetchMock)).resolves.toEqual({
          articleData: { figuresPdf: 'path-to-the-figures.pdf' },
          ok: true,
        })
      })
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

  describe('getArticleDoi', () => {
    it('it returns the expected DOI', () => {
      const mockData = '10.7554/eLife.30274'
      body.innerHTML = `<div itemprop="identifier"><meta content="https://registry.identifiers.org/registry/doi" /><span itemprop="value">${mockData}</span></div>`
      expect(dataProvider.getArticleDoi()).toEqual(mockData)
    })
  })
})
