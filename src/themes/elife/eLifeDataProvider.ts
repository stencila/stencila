interface ArticleData {
  status: number
}

export default {
  query: async (id: string, fetcher: Function): Promise<ArticleData> => {
    const data = await fetcher(`https://api.elifesciences.org/articles/${id}`)
    if (data.status === 404) {
      throw new ReferenceError(`Invalid eLife article id: ${id}`)
    }
    return Promise.resolve({ status: data.status })
  }
}
