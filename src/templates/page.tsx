import { graphql } from 'gatsby'
import { MDXRenderer } from 'gatsby-mdx'
import { Block, Column, Content, Tag, Title } from 'rbx'
import * as React from 'react'
import { CodeTabs } from '../components/CodeTabs'
import { LayoutProps, withLayout } from '../components/Layout'
import { SchemaTable } from '../components/SchemaTable'

interface DocumentationPageProps extends LayoutProps {
  data: {
    schema: any
    examples: any
    notes: any
  }
}

const Documentation = (props: DocumentationPageProps) => {
  const file = props.data.schema.edges[0].node
  if (!file.relativePath) {
    return <div>done</div>
  }

  const schema = require(`../../schema/${file.relativePath}`)
  const allOf = schema.allOf ? schema.allOf[1].properties : {}
  const anyOf = schema.anyOf ? schema.anyOf[1].properties : {}
  const properties = { ...schema.properties, ...allOf, ...anyOf }

  return (
    <Column className="is-clipped">
      <Title>
        {schema.title}
        <Tag color="info">{file.relativeDirectory}</Tag>
      </Title>
      <Title subtitle={true} as="h2">
        <code>{schema.$id}</code>
      </Title>

      <p>{schema.description}</p>

      <hr />

      <Column.Group>
        <Column size={6}>
          <SchemaTable schema={properties} />

          {props.data.notes && (
            <Block>
              <Title as="h3">Notes</Title>
              <Content>
                <MDXRenderer>{props.data.notes.code.body}</MDXRenderer>
              </Content>
            </Block>
          )}
        </Column>

        <Column size={6} paddingless={true}>
          <CodeTabs
            relativePath={file.relativePath}
            data={props.data.examples}
          />
        </Column>
      </Column.Group>
    </Column>
  )
}

export default withLayout(Documentation)

export const pageQuery = graphql`
  query DocumentationPage($fileRegex: String!, $relativePath: String!) {
    schema: allFile(
      filter: {
        relativePath: { eq: $relativePath }
        sourceInstanceName: { eq: "schemas" }
        internal: { mediaType: { eq: "text/yaml" } }
      }
    ) {
      edges {
        node {
          relativePath
          relativeDirectory
        }
      }
    }
    examples: allFile(
      filter: {
        relativeDirectory: { regex: $fileRegex }
        sourceInstanceName: { eq: "examples" }
      }
    ) {
      edges {
        node {
          relativePath
        }
      }
    }
    notes: mdx(fileAbsolutePath: { regex: $fileRegex }) {
      code {
        body
      }
    }
  }
`
