/* eslint-disable @typescript-eslint/ban-ts-comment */
import * as dataProvider from '../lib/dataProvider'
import Mock = jest.Mock

const body = document.body

const resetDom = (): void => {
  body.innerHTML = ''
}

describe('data Provider ', () => {
  afterEach(resetDom)

  describe("getting PDF URLs for an article's id", () => {
    let articleId: string
    let mockPdfUrlGetter: Mock

    beforeEach(() => {
      articleId = 'someId'
      mockPdfUrlGetter = jest.fn(
        // args are used when the mock is injected, so disabling check:
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        (id: string, pdfType: string): Promise<string> => {
          return Promise.resolve('theArticlePdfUri')
        }
      )
      jest.fn(mockPdfUrlGetter)
    })

    it('getArticlePdfUrl() requests the article PDF URL for the id', async (): Promise<
      unknown
    > => {
      await dataProvider.getArticlePdfUrl(articleId, mockPdfUrlGetter)
      const mockCalls = mockPdfUrlGetter.mock.calls
      expect(mockCalls.length).toBe(1)
      /* eslint-disable @typescript-eslint/no-unsafe-member-access */
      expect(mockCalls[0][0]).toBe(articleId)
      return expect(mockCalls[0][1]).toBe('article')
      /* eslint-enable @typescript-eslint/no-unsafe-member-access */
    })

    it('getFiguresPdfUrl() requests the figures PDF URL for the id', async (): Promise<
      unknown
    > => {
      await dataProvider.getFiguresPdfUrl(articleId, mockPdfUrlGetter)
      const mockCalls = mockPdfUrlGetter.mock.calls
      expect(mockCalls.length).toBe(1)
      /* eslint-disable @typescript-eslint/no-unsafe-member-access */
      expect(mockCalls[0][0]).toBe(articleId)
      return expect(mockCalls[0][1]).toBe('figures')
      /* eslint-enable @typescript-eslint/no-unsafe-member-access */
    })
  })

  describe('getArticleDoi', () => {
    it('it returns the expected DOI', () => {
      const mockData = '10.7554/eLife.30274'
      body.innerHTML = `<div itemprop="identifier"><meta content="https://registry.identifiers.org/registry/doi" /><span itemprop="value">${mockData}</span></div>`
      expect(dataProvider.getArticleDoi()).toEqual(mockData)
    })
  })

  describe('getArticleId', () => {
    it('it returns the expected eLife article Id', () => {
      const mockData = '30274'
      body.innerHTML = `<div itemprop="identifier"><meta content="https://registry.identifiers.org/registry/publisher-id" /><span itemprop="value">${mockData}</span></div>`
      expect(dataProvider.getArticleId()).toEqual(mockData)
    })
  })

  describe('getArticleTitle', () => {
    it('returns the correct text of a title', () => {
      const mockData =
        'Replication Study: Transcriptional amplification in tumor cells with elevated c-Myc'
      body.innerHTML = `<div itemprop="headline">${mockData}</div>`
      expect(dataProvider.getArticleTitle()).toEqual(mockData)
    })

    it('normalises any whitespace found', () => {
      const mockDataWithExtraWhitespace =
        'Replication    Study: Transcriptional \n  amplification in      \n\n  tumor cells with elevated c-Myc'
      const mockDataWithoutExtraWhitespace =
        'Replication Study: Transcriptional amplification in tumor cells with elevated c-Myc'
      body.innerHTML = `<div itemprop="headline">${mockDataWithExtraWhitespace}</div>`
      expect(dataProvider.getArticleTitle()).toEqual(
        mockDataWithoutExtraWhitespace
      )
    })

    it('omits any elements from the text it returns', () => {
      const mockDataWithMarkup =
        'Replication Study: Transcriptional amplification in <sup>tumor cells</sup> with elevated c-Myc'
      const mockDataWithoutMarkup =
        'Replication Study: Transcriptional amplification in tumor cells with elevated c-Myc'
      body.innerHTML = `<div itemprop="headline">${mockDataWithMarkup}</div>`
      expect(dataProvider.getArticleTitle()).toEqual(mockDataWithoutMarkup)
    })
  })
})
