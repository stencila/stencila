interface Response {
  ok: boolean
  articleData: { pdf: string; figuresPdf: string }
}

const getPdfUrl = async (id: string, pdfType: string): Promise<string> => {
  const allowedPdfTypes = ['article', 'figures']
  if (!allowedPdfTypes.includes(pdfType)) {
    return ''
  }
  const response = await query(id, window.fetch)
  if (pdfType === 'figures') {
    return Promise.resolve(response.articleData.figuresPdf)
  }
  return Promise.resolve(response.articleData.pdf)
}

export const getArticlePdfUrl = async (id: string): Promise<string> => {
  return getPdfUrl(id, 'article')
}

export const getFiguresPdfUrl = async (id: string): Promise<string> => {
  return getPdfUrl(id, 'figures')
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
