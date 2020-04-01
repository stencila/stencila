interface ArticleData {
  status: number
}

export default {
  query: (id: string, fetcher: Function): Promise<ArticleData> => {
    return fetcher(`https://api.elifesciences.org/articles/${id}`).then(
      (data: ArticleData) => {
        if (data.status === 404) {
          throw new ReferenceError(`Invalid eLife article id: ${id}`)
        }
        return { status: data.status }
      }
    )
  }
}
