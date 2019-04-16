const path = require('path')
const slash = require('slash')

exports.onCreateNode = ({ node, actions }) => {
  const { createNodeField } = actions

  if (node.internal.type === 'Json') {
    // GraphQL does not allow easily fetching all properties at once, so instead of
    // we stringify the contents and attach it as a custom field on the entry node.
    // This is later used to generate the schema properties table.
    createNodeField({
      node,
      name: `propertiesAsString`,
      value: JSON.stringify(node.properties)
    })

    createNodeField({
      node,
      name: `allOfAsString`,
      value: JSON.stringify(node.allOf)
    })

    createNodeField({
      node,
      name: `anyOfAsString`,
      value: JSON.stringify(node.anyOf)
    })
  }
}

// Implement the Gatsby API `createPages`.
// This is called after the Gatsby bootstrap is finished
// so you have access to any information necessary to
// programatically create pages.
exports.createPages = ({ graphql, actions }) => {
  const { createPage } = actions

  return new Promise((resolve, reject) => {
    graphql(`
      {
        allFile(
          filter: {
            sourceInstanceName: { eq: "schemas" }
            extension: { eq: "json" }
          }
        ) {
          edges {
            node {
              id
              name
              relativePath
              childJson {
                title
              }
            }
          }
        }
      }
    `)
      .then(result => {
        if (result.errors) {
          return reject(result.errors)
        }

        const schemas = result.data.allFile.edges.map(edge => edge.node)
        schemas.forEach(schema => {
          const title = schema.childJson
            ? schema.childJson.title
            : schema.name.split('.')[0]

          createPage({
            path: `/${title}`,
            component: slash(path.resolve('src/templates/page.tsx')),
            context: {
              fileRegex: `/${title}\./i`,
              relativePath: schema.relativePath
            }
          })
        })
      })
      .then(resolve)
  })
}
