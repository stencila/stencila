module.exports = {
  pathPrefix: '/schema',
  siteMetadata: {
    title: `Stencila Schema`
  },
  plugins: [
    // Plugin form MDX and Markdown transformation
    {
      resolve: `gatsby-mdx`,
      options: {
        extensions: [`.mdx`, `.md`],
        remarkPlugins: [require('remark-emoji')],
        gatsbyRemarkPlugins: [
          {
            resolve: `gatsby-remark-images`,
            options: {}
          },
          {
            resolve: `gatsby-remark-autolink-headers`,
            options: {
              icon: false
            }
          }
        ]
      }
    },

    `gatsby-transformer-yaml`,
    `gatsby-transformer-json`,

    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `schemas`,
        path: `${__dirname}/dist`
      }
    },
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `schemas`,
        path: `${__dirname}/dist`
      }
    },
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `examples`,
        path: `${__dirname}/examples`
      }
    },
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `data`,
        path: `${__dirname}/README.md`
      }
    },
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `data`,
        path: `${__dirname}/transports.png`
      }
    },

    // Add typescript stack into webpack
    `gatsby-plugin-typescript`,

    // This plugin takes your configuration and generates a
    // web manifest file so your website can be added to your
    // homescreen on Android.
    {
      resolve: `gatsby-plugin-manifest`,
      options: {
        name: `Stencila Schema`,
        short_name: `Stencila Schema`,
        start_url: `/`,
        background_color: `#f7f7f7`,
        theme_color: `#2568EF`,
        display: `minimal-ui`
      }
    },

    // This plugin generates a service worker and AppShell
    // html file so the site works offline and is otherwise
    // resistant to bad networks. Works with almost any
    // site!
    `gatsby-plugin-offline`
  ]
}
