interface Response {
  ok: boolean
  articleData: { pdf: string; figuresPdf: string }
}

export default async function (
  id: string,
  fetcher: WindowOrWorkerGlobalScope['fetch']
): Promise<Response> {
  const response = await fetcher(`https://api.elifesciences.org/articles/${id}`)
  if (response.ok === false) {
    throw new Error(`There was a problem getting article data for ${id}`)
  }
  const articleData = (await response.json()) as Response['articleData']
  return Promise.resolve({ ok: response.ok, articleData })
}
