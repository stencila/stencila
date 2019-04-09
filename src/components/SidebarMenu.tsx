import { graphql, Link, useStaticQuery } from 'gatsby'
import { Column, Menu, Tag } from 'rbx'
import * as React from 'react'

export const SidebarMenu = () => {
  const data = useStaticQuery(sidebarQuery)
  const schemas = data.allFile.edges.map(({ node }) => {
    return {
      ...node,
      title: node.name.split('.')[0],
      group: node.relativeDirectory
    }
  })
  return (
    <Column narrow={true} className="has-background-light">
      <Menu>
        <Menu.Label>Schema</Menu.Label>
        <Menu.List>
          {schemas.map(schema => (
            <Menu.List.Item as={Link} to={schema.title} key={schema.id}>
              {schema.title}{' '}
              {schema.group && <Tag color="info">{schema.group}</Tag>}
            </Menu.List.Item>
          ))}
        </Menu.List>
      </Menu>
    </Column>
  )
}

const sidebarQuery = graphql`
  query SidebarQuery {
    allFile(
      filter: {
        sourceInstanceName: { eq: "schemas" }
        internal: { mediaType: { eq: "text/yaml" } }
      }
      sort: { fields: [name] }
    ) {
      edges {
        node {
          id
          name
          relativePath
          relativeDirectory
        }
      }
    }
  }
`
