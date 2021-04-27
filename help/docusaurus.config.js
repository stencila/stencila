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
      items: [
        {
          type: 'doc',
          docId: 'welcome',
          position: 'left',
          label: 'Welcome',
        },
        {
          href: 'https://github.com/stencila/stencila',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    algolia: {
      apiKey: 'YOUR_API_KEY',
      indexName: 'YOUR_INDEX_NAME',
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Tutorials',
          items: [
            {
              label: 'README',
              to: '/tutorials/README',
            },
          ],
        },
        {
          title: 'Guides',
          items: [
            {
              label: 'Organizations',
              to: '/guides/organizations',
            },
          ],
        },
        {
          title: 'Reference',
          items: [
            {
              label: 'README',
              to: '/reference/README',
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
          // Please change this to your repo.
          editUrl:
            'https://github.com/facebook/docusaurus/edit/master/website/',
        },
        blog: {
          showReadingTime: true,
          // Please change this to your repo.
          editUrl:
            'https://github.com/facebook/docusaurus/edit/master/website/blog/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
}
