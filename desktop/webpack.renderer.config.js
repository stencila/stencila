const CopyWebpackPlugin = require('copy-webpack-plugin')
const CspHtmlWebpackPlugin = require('csp-html-webpack-plugin')
const HtmlInsertTagWebpackPlugin = require('html-insert-tag-webpack-plugin')
const path = require('path')
const plugins = require('./webpack.plugins')
const webpack = require('webpack')
const rules = require('./webpack.rules')

rules.push({
  test: /\.css$/,
  use: [{ loader: 'style-loader' }, { loader: 'css-loader' }],
})

module.exports = {
  module: {
    rules,
  },
  plugins: [
    ...plugins,
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify(
        process.env.NODE_ENV ?? 'development'
      ),
      'process.env.SENTRY_DSN': JSON.stringify(process.env.SENTRY_DSN),
      'process.type': '"renderer"',
    }),
    new HtmlInsertTagWebpackPlugin([
      {
        tagName: 'base',
        inject: {
          tagName: 'head',
          location: 'after',
        },
        attributes: {
          href: 'stencila://rse',
        },
      },
    ]),
    new CspHtmlWebpackPlugin(
      {
        'base-uri': "'self' stencila://rse",
        'default-src': 'stencila://rse',
        'script-src': ["'self'"],
        'img-src': ["'self'", 'data:', 'local:'],
        'style-src': [
          "'self'",
          "'unsafe-inline'",
          'https://unpkg.com/',
          'https://fonts.googleapis.com/',
        ],
        'connect-src': "'self'",
        'font-src': [
          "'self'",
          'https://fonts.gstatic.com/',
          'https://unpkg.com/',
        ],
      },
      {
        hashEnabled: {
          'script-src': true,
          'style-src': true,
        },
        nonceEnabled: {
          'script-src': true,
          'style-src': false,
        },
      }
    ),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: path.resolve(__dirname, 'www', 'build'),
          to: 'build',
        },
        {
          from: path.resolve(__dirname, 'www', 'assets'),
          to: 'assets',
        },
        {
          from: path.resolve(__dirname, 'www', 'manifest.json'),
          to: '.',
        },
      ],
    }),
  ],
  resolve: {
    mainFields: ['browser', 'module', 'main'],
    extensions: ['.js', '.mjs', '.ts', '.css'],
  },
}
