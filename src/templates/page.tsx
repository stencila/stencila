import { graphql } from 'gatsby'
import { MDXRenderer } from 'gatsby-mdx'
import { Block, Column, Content, Title } from 'rbx'
import * as React from 'react'
import { CodeTabs } from '../components/CodeTabs'
import { LayoutProps, withLayout } from '../components/Layout'
import { SchemaTable } from '../components/SchemaTable'

interface DocumentationPageProps extends LayoutProps {
  data: {
    post: any
    examples: any
    mdx: any
  }
}

const Documentation = (props: DocumentationPageProps) => {
  const post = props.data.post
  if (!post.title) {
    return <div>done</div>
  }

  const schema = require(`../../schema/${post.title}.schema.yaml`)

  const allOf = schema.allOf ? schema.allOf[1].properties : {}
  const anyOf = schema.anyOf ? schema.anyOf[1].properties : {}
  const _schema = { ...schema.properties, ...allOf, ...anyOf }

  return (
    <Column className="is-clipped">
      <Title>{post.title}</Title>
      <Title subtitle={true} as="h2">
        <code>{post._id}</code>
      </Title>

      <p>{post.description}</p>

      <hr />

      <Column.Group>
        <Column size={6}>
          <SchemaTable schema={_schema} />

          {props.data.mdx && (
            <Block>
              <Title as="h3">Notes</Title>
              <Content>
                <MDXRenderer>{props.data.mdx.code.body}</MDXRenderer>
              </Content>
            </Block>
          )}
        </Column>

        <Column size={6} paddingless={true}>
          <CodeTabs title={post.title} data={props.data.examples} />
        </Column>
      </Column.Group>
    </Column>
  )
}

export default withLayout(Documentation)

export const pageQuery = graphql`
  query DocumentationPage($slug: String!, $title: String!) {
    post: schemaYaml(fields: { slug: { eq: $slug } }) {
      title
      description
      _id
      fields {
        slug
      }
    }
    examples: allFile(
      filter: {
        name: { regex: $title }
        sourceInstanceName: { eq: "examples" }
      }
    ) {
      edges {
        node {
          relativePath
        }
      }
    }
    mdx: mdx(fileAbsolutePath: { regex: $title }) {
      code {
        body
      }
    }
  }
`
