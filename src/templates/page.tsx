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

  const schema = file.childJson

  const properties = {
    ...JSON.parse(schema.fields.allOfAsString),
    ...JSON.parse(schema.fields.anyOfAsString),
    ...JSON.parse(schema.fields.propertiesAsString)
  }

  return (
    <Column className="is-clipped">
      <Title>
        {schema.title}
        {schema.category && schema.category !== '.' && (
          <Tag color="info">{schema.category}</Tag>
        )}
      </Title>

      <Title subtitle={true} as="h2">
        <code>{schema._id}</code>
      </Title>

      <p>{schema.description}</p>

      <hr />

      <Column.Group>
        <Column size={6}>
          {props.data.notes && (
            <>
              <Block>
                <Content>
                  <MDXRenderer>{props.data.notes.code.body}</MDXRenderer>
                </Content>
              </Block>

              <hr />
            </>
          )}

          <Title as="h3">{schema.title} Properties</Title>

          <SchemaTable schema={properties} />
        </Column>

        {props.data.examples.edges.length > 0 && (
          <Column size={6} paddingless={true}>
            <CodeTabs data={props.data.examples} />
          </Column>
        )}
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
        extension: { eq: "json" }
      }
    ) {
      edges {
        node {
          relativePath
          relativeDirectory
          childJson {
            _id
            fields {
              allOfAsString
              anyOfAsString
              propertiesAsString
            }
            category
            description
            role
            title
          }
        }
      }
    }
    examples: allFile(
      filter: {
        relativePath: { regex: $fileRegex }
        sourceInstanceName: { eq: "examples" }
      }
      sort: { fields: [relativePath] }
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
