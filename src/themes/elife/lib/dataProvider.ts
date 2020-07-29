import { first, text } from '../../../util'
import { articleData } from './query'

interface PdfUrlGetter {
  (article: articleData, pdfType: string): string
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

const getPdfUrl: PdfUrlGetter = (
  article: articleData,
  pdfType: string
): string => {
  const allowedPdfTypes = ['article', 'figures']
  if (!allowedPdfTypes.includes(pdfType)) {
    throw new Error(
      `Requested Invalid PDF type: "${pdfType}", must be one of ${allowedPdfTypes.join(
        ', '
      )}.`
    )
  }
  return pdfType === 'figures' ? article.figuresPdf : article.pdf
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

export const getArticlePdfUrl = (
  article: articleData,
  pdfUrlGetter: PdfUrlGetter = getPdfUrl
): string => {
  return pdfUrlGetter(article, 'article')
}

export const getFiguresPdfUrl = (
  article: articleData,
  pdfUrlGetter: PdfUrlGetter = getPdfUrl
): string => {
  return pdfUrlGetter(article, 'figures')
}
