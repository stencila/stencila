const path = require('path')
const slash = require('slash')

// Implement the Gatsby API `createPages`.
// This is called after the Gatsby bootstrap is finished
// so you have access to any information necessary to
// programatically create pages.
exports.createPages = ({ graphql, actions }) => {
  const { createPage } = actions

  return new Promise((resolve, reject) => {
    graphql(`
      {
        allFile(filter: { sourceInstanceName: { eq: "schemas" } }) {
          edges {
            node {
              id
              name
              relativePath
              childDistJson {
                title
              }
            }
          }
        }
      }
    `).then(result => {
      if (result.errors) {
        return reject(result.errors)
      }

      const schemas = result.data.allFile.edges.map(edge => edge.node)
      schemas.forEach(schema => {
        const title = schema.childDistJson
          ? schema.childDistJson.title
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

      resolve()
    })
  })
}
