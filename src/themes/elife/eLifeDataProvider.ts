interface ArticleData {
  status: number
}

export default {
  query: (id: string, fetcher: Function): Promise<ArticleData> => {
    return fetcher(`https://api.elifesciences.org/articles/${id}`).then(
      (data: ArticleData) => {
        return { status: data.status }
      }
    )
  }
}
