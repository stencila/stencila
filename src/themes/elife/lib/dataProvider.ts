import { first, text } from '../../../util'

interface Response {
  ok: boolean
  articleData: { pdf: string; figuresPdf: string }
}

interface PdfUrlGetter {
  (id: string, pdfType: string): Promise<string>
}

const normaliseWhitespace = (txt: string): string => {
  return txt.replace(/\n/, ' ').replace(/ \s+|\n+/g, ' ')
}

const getNormalisedTextFromElement = (selector: string): string => {
  const target = first(selector)
  if (target !== null) {
    const sourceText = text(target)
    if (sourceText !== null) {
      return normaliseWhitespace(sourceText)
    }
  }
  return ''
}

const getPdfUrl: PdfUrlGetter = async (
  id: string,
  pdfType: string
): Promise<string> => {
  const allowedPdfTypes = ['article', 'figures']
  if (!allowedPdfTypes.includes(pdfType)) {
    throw new Error(
      `Requested Invalid PDF type: "${pdfType}", must be one of ${allowedPdfTypes.join(
        ', '
      )}.`
    )
  }
  const response = await query(id, window.fetch)
  if (pdfType === 'figures') {
    return Promise.resolve(response.articleData.figuresPdf)
  }
  return Promise.resolve(response.articleData.pdf)
}

export const getArticleId = (): string => {
  return getNormalisedTextFromElement(
    ':--identifier meta[content="https://registry.identifiers.org/registry/publisher-id"] ~ [itemprop="value"]'
  )
}

export const getArticleDoi = (): string => {
  return getNormalisedTextFromElement(
    ':--identifier meta[content="https://registry.identifiers.org/registry/doi"] ~ [itemprop="value"]'
  )
}

export const getArticleTitle = (): string => {
  return getNormalisedTextFromElement(':--title')
}

export const getArticlePdfUrl = async (
  id: string,
  pdfUrlGetter: PdfUrlGetter = getPdfUrl
): Promise<string> => {
  return pdfUrlGetter(id, 'article')
}

export const getFiguresPdfUrl = async (
  id: string,
  pdfUrlGetter: PdfUrlGetter = getPdfUrl
): Promise<string> => {
  return pdfUrlGetter(id, 'figures')
}

export const query = async (
  id: string,
  fetcher: Function
): Promise<Response> => {
  const response = await fetcher(`https://api.elifesciences.org/articles/${id}`)
  if (response.ok === false) {
    throw new Error(`There was a problem getting article data for ${id}`)
  }
  const articleData = await response.json()
  return Promise.resolve({ ok: response.ok, articleData })
}
