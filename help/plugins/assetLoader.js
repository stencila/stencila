// Custom Docusaurus plugin to load `.cast` files as static file assets.
// This allows usage in Markdown/MDX files like so:
//
// ```md
// import AsciinemaPlayer from '../../../src/components/asciinema/player'
// import upgradeCast from './01-upgrading.cast'
//
// # My document title
//
// <AsciinemaPlayer src={upgradeCast} />
//
// Some paragraph text
// ```

const { configureWebpack } = require('@docusaurus/core/lib/webpack/utils')

const assetsRelativeRoot = 'assets/'

module.exports = function (context, options) {
  return {
    name: 'asset-loader',
    configureWebpack(config) {
      return {
        module: {
          rules: [
            {
              test: /\.(cast)$/i,
              use: [
                {
                  loader: 'file-loader',
                  options: { name: `${assetsRelativeRoot}[name]-[hash].[ext]` },
                },
              ],
            },
          ],
        },
      }
    },
  }
}
