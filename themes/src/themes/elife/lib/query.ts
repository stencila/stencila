export interface articleData {
  pdf: string
  figuresPdf?: string
  copyright: { license: string }
}

interface Response {
  ok: boolean
  article: articleData
}

export default async function (
  id: string,
  fetcher: WindowOrWorkerGlobalScope['fetch']
): Promise<Response> {
  const response = await fetcher(`https://api.elifesciences.org/articles/${id}`)
  if (response.ok === false) {
    throw new Error(`There was a problem getting article data for ${id}`)
  }
  const article = (await response.json()) as articleData
  return Promise.resolve({ ok: response.ok, article })
}
