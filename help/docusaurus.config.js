/** @type {import('@docusaurus/types').DocusaurusConfig} */
const path = require('path')

const baseUrl = '/'

const commonPluginContentDocsOptions = {
  editUrl: 'https://github.com/stencila/stencila/edit/master/help',
  showLastUpdateAuthor: true,
  showLastUpdateTime: true,
}

module.exports = {
  title: 'Stencila Help',
  url: 'https://stencila.github.io',
  baseUrl: baseUrl,
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',
  organizationName: 'stencila',
  projectName: 'stencila',
  themeConfig: {
    // Default image used for meta tags e.g og:image and twitter:image
    // Can't be an SVG.
    image: 'img/stencila.png',
    navbar: {
      title: 'Help',
      logo: {
        alt: 'Stencila Logo',
        src: 'img/stencilaLogo.svg',
        srcDark: 'img/stencilaLogoDarkBG.svg',
      },
      items: [
        { to: 'tutorials', label: 'Tutorials', position: 'right' },
        { to: 'guides', label: 'Guides', position: 'right' },
        { to: 'demos', label: 'Demos', position: 'right' },
        { to: 'references', label: 'References', position: 'right' },
      ],
    },
    algolia: {
      apiKey: '8f59224393d852be19f51abd67a9fbbb',
      indexName: 'stenci',
    },
    colorMode: {
      defaultMode: 'light',
      // Respect user's system preferences for `prefers-color-scheme` media-query
      respectPrefersColorScheme: true,
      switchConfig: {
        darkIcon: '🌙',
        darkIconStyle: {
          fontSize: '80%',
        },
        lightIcon: '☀️',
        lightIconStyle: {
          fontSize: '80%',
        },
      },
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Hub',
          items: [
            {
              label: 'Gallery',
              href: 'https://hub.stenci.la',
            },
            {
              label: 'Sign in',
              href: 'https://hub.stenci.la/me/signin/',
            },
            {
              label: 'Sign up',
              href: 'https://hub.stenci.la/me/signup/',
            },
          ],
        },
        {
          title: 'Download',
          items: [
            {
              label: 'CLI',
              href:
                'https://github.com/stencila/stencila/tree/master/cli#-install',
            },
            {
              label: 'Desktop',
              href:
                'https://github.com/stencila/stencila/tree/master/desktop#-install',
            },
          ],
        },
        {
          title: 'Tools & Plugins',
          items: [
            {
              label: 'Schema',
              href: 'https://github.com/stencila/schema#readme',
            },
            {
              label: 'Encoda',
              href: 'https://github.com/stencila/encoda#readme',
            },
            {
              label: 'Thema',
              href: 'https://github.com/stencila/thema#readme',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'Discord',
              href: 'https://discord.gg/pzUz8R3',
            },
            {
              label: 'GitHub',
              href: 'https://github.com/stencila/stencila/discussions',
            },
            {
              label: 'Twitter',
              href: 'https://twitter.com/stencila',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Stencila`,
    },
  },
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
  plugins: [
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'tutorials',
        path: 'tutorials',
        routeBasePath: 'tutorials',
        sidebarPath: require.resolve('./tutorials/sidebars.js'),
        ...commonPluginContentDocsOptions,
      },
    ],
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'guides',
        path: 'guides',
        routeBasePath: 'guides',
        sidebarPath: require.resolve('./guides/sidebars.js'),
        ...commonPluginContentDocsOptions,
      },
    ],
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'demos',
        path: 'demos',
        routeBasePath: 'demos',
        sidebarPath: require.resolve('./demos/sidebars.js'),
        ...commonPluginContentDocsOptions,
      },
    ],
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'references',
        path: 'references',
        routeBasePath: 'references',
        sidebarPath: require.resolve('./references/sidebars.js'),
        ...commonPluginContentDocsOptions,
      },
    ],
    [
      '@docusaurus/plugin-client-redirects',
      // Note that these redirects do not work on the development server, only
      // by creating index.html files for the prod build
      {
        redirects: [
          // Redirects from some of the circa 2020-21 Intercom-hosted
          // help articles and collections
          {
            from: '/en/articles/4184684-enriching-an-elife-article',
            to: '/docs/tutorials/enriching-an-elife-article',
          },
          {
            from: '/en/collections/2549573-formats',
            to: '/docs/guides',
          },
          {
            from: '/en/collections/2549573-formats-syntax-references',
            to: '/docs/guides',
          },
          {
            from: '/en/articles/4458566-r-markdown',
            to: '/docs/guides/formats/rmarkdown',
          },
          {
            from: '/en/articles/4624378-jupyter-notebooks',
            to: '/docs/guides/formats/jupyter-notebooks',
          },
          {
            from: '/en/collections/2378614-stencila-for-gsuite',
            to: '/docs/tutorials',
          },
          {
            from: '/en/articles/4857017-getting-started',
            to: '/docs/tutorials',
          },
          {
            from: '/en/articles/4857019-installing-stencila-for-google-docs',
            to: '/docs/tutorials',
          },
          {
            from: '/en/articles/4857020-add-on-interface-overview',
            to: '/docs/tutorials',
          },
        ],
      },
    ],
    path.resolve(__dirname, './src/plugins/assetLoader'),
  ],
  scripts: [
    {
      src: `${baseUrl}asciinema-player.js`,
    },
  ],
}
