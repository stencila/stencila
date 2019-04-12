import { graphql, Link, useStaticQuery } from 'gatsby'
import { Column, Menu, Tag } from 'rbx'
import * as React from 'react'

export const SidebarMenu = () => {
  const data = useStaticQuery(sidebarQuery)
  const schemas = data.allDistJson.edges.map(({ node }) => {
    return {
      ...node,
      title: node.title || '',
      category: node.category
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
              {schema.category && schema.category !== '.' && (
                <Tag color="info">{schema.category}</Tag>
              )}
            </Menu.List.Item>
          ))}
        </Menu.List>
      </Menu>
    </Column>
  )
}

const sidebarQuery = graphql`
  query SidebarQuery {
    allDistJson(filter: { title: { ne: null } }, sort: { fields: [title] }) {
      edges {
        node {
          id
          title
          category
        }
      }
    }
  }
`
