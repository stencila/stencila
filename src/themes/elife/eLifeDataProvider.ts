interface Response {
  status: number
}

export default {
  query: async (id: string, fetcher: Function): Promise<Response> => {
    const response = await fetcher(
      `https://api.elifesciences.org/articles/${id}`
    )
    if (response.status === 404) {
      throw new ReferenceError(`Invalid eLife article id: ${id}`)
    }
    return Promise.resolve({ status: response.status })
  }
}
