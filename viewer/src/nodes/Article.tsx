import * as schema from '@stencila/schema'
import { BlockContentArray } from './BlockContent'
import { For, Switch, Match } from 'solid-js'
import { ContentArray } from './Content'
import { CreativeWork } from '@stencila/schema'

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
          <ContentArray
            nodes={props.description as schema.InlineContent[]}
          ></ContentArray>
        </Match>
      </Switch>
    </section>
  )
}

export function ArticleReferences(props: {
  references: Exclude<schema.Article['references'], undefined>
}) {
  return (
    <section attr:data-itemprop="data-references">
      <h2 itemtype="http://schema.stenci.la/Heading">References</h2>
      <ol>
        <For each={props.references}>
          {(reference, index) => (
            <Switch>
              <Match when={typeof reference === 'string'}>
                <ArticleReferenceString
                  reference={reference as string}
                  index={index()}
                />
              </Match>
              <Match when={schema.isCreativeWork(reference)}>
                <ArticleReferenceCreativeWork
                  reference={reference as CreativeWork}
                  index={index()}
                />
              </Match>
            </Switch>
          )}
        </For>
      </ol>
    </section>
  )
}

export function ArticleReferenceString(props: {
  reference: string
  index: number
}) {
  return (
    <li itemprop="citation" id={`ref${props.index}`}>
      {props.reference}
    </li>
  )
}

export function ArticleReferenceCreativeWork(props: {
  reference: schema.CreativeWork
  index: number
}) {
  return (
    <li
      itemprop="citation"
      itemtype="http://schema.org/Article"
      itemscope
      id={props.reference.id ? props.reference.id : `ref${props.index}`}
    >
      {props.reference.authors && (
        <ol data-itemprop="authors">TODO: authors</ol>
      )}
      {props.reference.title && (
        <span itemprop="headline">{props.reference.title}</span>
      )}
    </li>
  )
}
