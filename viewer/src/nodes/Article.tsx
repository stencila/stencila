import * as schema from '@stencila/schema'
import { BlockContentArray } from './BlockContent'
import { For, Switch, Match } from 'solid-js'
import { ContentArray } from './Content'

export function Article(props: { node: schema.Article }) {
  return (
    <article itemtype="http://schema.org/Article" itemscope>
      {props.node.title && <h1 itemprop="headline">{props.node.title}</h1>}

      {props.node.authors && (
        <ArticleAuthors authors={props.node.authors}></ArticleAuthors>
      )}

      {props.node.description && (
        <ArticleDescription
          description={props.node.description}
        ></ArticleDescription>
      )}

      {(props.node.content ?? []).length ? (
        <BlockContentArray nodes={props.node.content}></BlockContentArray>
      ) : (
        <div class="empty">Empty article</div>
      )}

      {props.node.references && (
        <ArticleReferences
          references={props.node.references}
        ></ArticleReferences>
      )}
    </article>
  )
}

export function ArticleAuthors(props: {
  authors: Exclude<schema.Article['authors'], undefined>
}) {
  return (
    <ol data-itemprop="authors">
      <For each={props.authors}>
        {(author) => <ArticleAuthor author={author} />}
      </For>
    </ol>
  )
}

export function ArticleAuthor(props: {
  author: schema.Person | schema.Organization
}) {
  return <li>TODO</li>
}

export function ArticleDescription(props: {
  description: Exclude<schema.Article['description'], undefined>
}) {
  return (
    <section data-itemprop="description">
      <h1>Abstract</h1>
      <Switch>
        <Match when={typeof props.description == 'string'}>
          {props.description as string}
        </Match>
        <Match when={typeof props.description != 'string'}>
          <ContentArray nodes={props.description as schema.InlineContent[] }></ContentArray>
        </Match>
      </Switch>
    </section>
  )
}

export function ArticleReferences(props: {
  references: Exclude<schema.Article['references'], undefined>
}) {
  return (
    <>
      <h1>References</h1>
      <ol>
        <For each={props.references}>
          {(reference) => <ArticleReference reference={reference} />}
        </For>
      </ol>
    </>
  )
}

export function ArticleReference(props: {
  reference: string | schema.CreativeWork
}) {
  return <li>TODO</li>
}
