import { graphql, Link, useStaticQuery } from 'gatsby'
import { Column, Menu } from 'rbx'
import * as React from 'react'

export const SidebarMenu = () => {
  const data = useStaticQuery(sidebarQuery)

  return (
    <Column narrow={true} className="has-background-light">
      <Menu>
        <Menu.Label>Schema</Menu.Label>
        <Menu.List>
          {data.allSchemaYaml.edges.map(({ node }) => (
            <Menu.List.Item as={Link} to={node.fields.slug} key={node.id}>
              {node.title}
            </Menu.List.Item>
          ))}
        </Menu.List>
      </Menu>
    </Column>
  )
}

const sidebarQuery = graphql`
  query SidebarQuery {
    allSchemaYaml {
      edges {
        node {
          id
          title
          fields {
            slug
          }
        }
      }
    }
  }
`
