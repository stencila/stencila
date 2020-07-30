/* eslint-disable @typescript-eslint/ban-ts-comment */
import query from '../lib/query'

interface Response {
  ok: boolean
  json: () => Promise<unknown>
}

describe('query', () => {
  describe('successfully querying a valid article id', () => {
    it('does not throw', async () => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve(),
        })
      await expect(
        // @ts-expect-error
        query('validArticleId', fetchMock)
      ).resolves.not.toThrow()
    })

    it('it exposes the url of the article PDF', async () => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ pdf: 'path-to-the.pdf' }),
        })
      // @ts-expect-error
      await expect(query('someId', fetchMock)).resolves.toEqual({
        article: { pdf: 'path-to-the.pdf' },
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
      // @ts-expect-error
      await expect(query('someId', fetchMock)).resolves.toEqual({
        article: { figuresPdf: 'path-to-the-figures.pdf' },
        ok: true,
      })
    })
  })

  it('it exposes the copyright license', async () => {
    const fetchMock = (): Promise<Response> =>
      Promise.resolve({
        ok: true,
        json: () =>
          Promise.resolve({
            copyright: {
              license: 'CC',
            },
          }),
      })
    // @ts-expect-error
    await expect(query('someId', fetchMock)).resolves.toEqual({
      article: { copyright: { license: 'CC' } },
      ok: true,
    })
  })

  describe('being given an invalid article id', () => {
    it('throws an ReferenceError', async () => {
      const fetchMock = (): Promise<Response> =>
        Promise.resolve({ ok: false, json: () => Promise.resolve() })
      await expect(
        // @ts-expect-error
        query('invalidArticleId', fetchMock)
      ).rejects.toThrow(
        new Error(
          `There was a problem getting article data for invalidArticleId`
        )
      )
    })
  })
})
