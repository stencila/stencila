module.exports = {
  pathPrefix: '/schema',
  siteMetadata: {
    title: `Stencila Schema`
  },
  plugins: [
    `gatsby-mdx`,
    `gatsby-transformer-yaml`,
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `data`,
        path: `${__dirname}/schema`
      }
    },
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `examples`,
        path: `${__dirname}/examples`
      }
    },

    // Parse JSON files
    // `gatsby-transformer-json`,

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
