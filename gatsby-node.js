const path = require("path");
const slash = require("slash");

// Create slugs for files.
// Slug will used for blog page path.
exports.onCreateNode = ({ node, actions }) => {
  const { createNodeField } = actions;
  let slug;
  switch (node.internal.type) {
    case `SchemaYaml`: {
      slug = `/docs/${node.title}/`;
      break;
    }
  }

  if (slug) {
    createNodeField({ node, name: `slug`, value: slug });
  }
};

// Implement the Gatsby API `createPages`.
// This is called after the Gatsby bootstrap is finished
// so you have access to any information necessary to
// programatically create pages.
exports.createPages = ({ graphql, actions }) => {
  const { createPage } = actions;

  return new Promise((resolve, reject) => {
    graphql(
      `
        {
          allSchemaYaml {
            edges {
              node {
                title
                id
                value
                description
                fields {
                  slug
                }
              }
            }
          }
        }
      `
    ).then(result => {
      if (result.errors) {
        return reject(result.errors);
      }
      const posts = result.data.allSchemaYaml.edges.map(p => p.node);

      posts.forEach(post => {
        createPage({
          path: post.fields.slug,
          component: slash(path.resolve("src/templates/page.tsx")),
          context: {
            title: `/${post.title}/i`,
            slug: post.fields.slug
          }
        });
      });

      resolve();
    });
  });
};
