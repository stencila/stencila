/* eslint-disable @typescript-eslint/camelcase */
import { dump, read, write } from '@stencila/encoda'
import { Article } from '@stencila/schema'
import glob from 'glob'
import fetch from 'node-fetch'
import path from 'path'

const mdPaths = glob.sync(path.join(__dirname, '/hub/**/*.md'))
const authToken = process.env.INTERCOM_AUTH_TOKEN

interface IntercomPartialArticle {
  id?: string
  author_id: string
  body?: string
  description?: string
  parent_id?: string
  parent_type?: 'collection' | 'section'
  state?: 'draft' | 'published'
  title: string
  translated_content?: Record<string, Article>
}

interface IntercomArticle extends Required<IntercomPartialArticle> {
  type: 'article'
  id: string
}

/**
 * Make an API request to Intercom servers.
 * If the article in the payload contains an `id`, update it, otherwise create a new one.
 */
const upsertArticle = (
  payload: IntercomPartialArticle
): Promise<IntercomArticle> => {
  const articlesUrl = 'https://api.intercom.io/articles'
  const url =
    payload.id === undefined ? articlesUrl : `${articlesUrl}/${payload.id}`

  const method = payload.id === undefined ? 'POST' : 'PUT'

  return fetch(url, {
    method,
    headers: {
      Authorization: `Bearer ${authToken}`,
      Accept: 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(payload)
  })
    .then(res => res.json())
    .then(res => {
      if (res.errors !== undefined && res.errors.length > 0) {
        console.log(res)
        throw new Error(JSON.stringify(res))
      }
      return res
    })
}

/**
 * Assume that Paragraphs containing a single Link element should be stylized as buttons
 */
const buttonifyLinks = (article: string): string => {
  const buttonLinkRegEx = /<p><a ([^>]+)>[^<]+?<\/a><\/p>/g
  return article.replace(buttonLinkRegEx, (match, attrs) =>
    match
      .replace('<p>', '<div class="intercom-align-center">')
      .replace(attrs, `${attrs} class="intercom-h2b-button"`)
      .replace('</p>', '</div>')
  )
}

type HelpArticle = Article & {
  description?: string
  state?: string
  parent_type?: string
  parent_id?: string
}

/**
 * Read a Markdown file, construct an Intercom Article API compatible payload.
 * If the article is already present on Intercom update it, otherwise create a new article.
 */
const postArticle = async (
  filePath: string,
  authorId: string,
  index: number
): Promise<IntercomPartialArticle> => {
  const article = (await read(filePath)) as HelpArticle
  const bodyRaw = await dump(article, 'html', {
    isStandalone: false,
    theme: 'stencila'
  })

  // Process the ingested HTML contents with various adjustments
  // TODO: Read list of `authors` in frontmatter and append as list at end of article
  const body = buttonifyLinks(bodyRaw)

  const articlePayload: IntercomPartialArticle = {
    author_id: authorId,
    body: body,
    description: article.description,
    id: article.id,
    parent_id: article.parent_id,
    parent_type:
      article.parent_type === 'collection' || article.parent_type === 'section'
        ? article.parent_type
        : undefined,
    state: article.state === 'published' ? 'published' : 'draft',
    title: typeof article.title === 'string' ? article.title : ''
  }

  try {
    if (article.id === undefined) {
      console.log(
        `🎉 ${index}/${mdPaths.length} Creating article: "${article.title}"`
      )
      // This is a new article that doesn't exist on Intercom yet.
      // Post to Intercom, and update the MD file with the returned ID
      const res = await upsertArticle(articlePayload)
      await write({ ...article, id: parseInt(res.id, 10) }, filePath, {
        format: 'md',
        theme: 'stencila'
      })
    } else {
      console.log(
        `🔄 ${index}/${mdPaths.length} Updating article: "${article.title} (#${article.id})"`
      )
      await upsertArticle(articlePayload)
    }
  } catch (err) {
    console.log(err)
  }

  return articlePayload
}

/**
 * Iterate through all Markdown help articles and submit them to Intercom.
 */
const updateAllArticles = async (): Promise<void> => {
  const authorId = process.env.INTERCOM_AUTHOR_ID

  if (!authorId) {
    throw new Error('Author ID environment variable is missing')
  }

  if (!authToken) {
    throw new Error('Intercom API auth token environment variable is missing')
  }

  let progressMeter = 1
  for (const file of mdPaths) {
    await postArticle(file, authorId, progressMeter).catch(e =>
      console.error(e)
    )
    progressMeter++
  }
}

updateAllArticles().catch(console.error)
