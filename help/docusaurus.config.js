/** @type {import('@docusaurus/types').DocusaurusConfig} */
module.exports = {
  title: 'Stencila Help',
  url: 'https://stencila.github.io/stencila',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',
  organizationName: 'stencila',
  projectName: 'stencila',
  themeConfig: {
    navbar: {
      title: 'Stencila Help',
      logo: {
        alt: 'Stencila Logo',
        src: 'img/logo.svg',
      },
    },
    algolia: {
      apiKey: 'YOUR_API_KEY',
      indexName: 'YOUR_INDEX_NAME',
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
              href: 'https://hub.stenci.la/me/signin',
            },
            {
              label: 'Sign up',
              href: 'https://hub.stenci.la/me/signup',
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
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/stencila/stencila/edit/master/help/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
}
