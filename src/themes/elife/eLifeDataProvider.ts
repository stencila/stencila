interface Response {
  ok: boolean
  articleData: { pdf: string }
}

export default {
  query: async (id: string, fetcher: Function): Promise<Response> => {
    const response = await fetcher(
      `https://api.elifesciences.org/articles/${id}`
    )
    if (response.ok === false) {
      throw new Error(`There was a problem getting article data for ${id}`)
    }
    const articleData = await response.json()
    return Promise.resolve({ ok: response.ok, articleData })
  }
}
