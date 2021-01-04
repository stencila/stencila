const globby = require('globby')
const path = require('path')
const MiniCssExtractPlugin = require('mini-css-extract-plugin')
const HtmlWebpackPlugin = require('html-webpack-plugin')
const { DefinePlugin } = require('webpack')

const contentSource = 'src'
const ASSET_PATH = process.env.ASSET_PATH || '/'

// Convert absolute filepaths to project relative ones to use as
// output destinations.
const makeRelativePath = (filepath) =>
  path.relative(path.join(__dirname, contentSource), filepath)

// Strip `/src` from output destination pathnames.
// Otherwise Webpack outputs files at `/dist/src/*`
const fileLoaderOutputPath = ({ filename }) => {
  return filename.replace(`${contentSource}/`, '')
}

const browserEntries = ['./src/**/*.{css,ts,ttf,woff,woff2}', '!**/lib/**/*.ts']

const libEntries = ['./src/lib/**/*.ts']

module.exports = (env = {}, { mode }) => {
  // Build target, can be one of: [`browser` | 'lib' | 'docs']
  const target = env.target || 'browser'
  const isDocs = env.target === 'docs'
  const isDevelopment = mode === 'development'
  const contentBase = isDocs ? 'docs' : 'dist'

  const entries = [
    ...(target === 'lib' ? libEntries : browserEntries),
    // Don’t compile test files for package distribution
    '!**/*.{d,test}.ts',
    // These files make use of Node APIs, and do not need to be packaged for Browser targets
    '!**/scripts/*.ts',
    '!**/extensions/math/update.ts',
    '!**/extensions/extensions.ts',
    // Don’t build HTML demo files for package distribution
    ...(isDocs || isDevelopment
      ? [
          './src/**/*.{jpg,png,gif,tsx,html}',
          // Template are used as basis for HtmlWebpackPlugin, and should not be used as an entry points
          '!./src/demo/templates/*',
        ]
      : ['!**/*.html', '!./src/demo/**/*', '!./src/examples/*']),
  ]

  const entry = globby.sync(entries).reduce(
    (files, file) => ({
      ...files,
      [makeRelativePath(file).replace(/.ts$/, '').replace(/.css$/, '')]: file,
    }),
    {}
  )

  // Only generate HTML files for documentation builds, and local development
  const docsPlugins =
    isDocs || isDevelopment
      ? [
          new HtmlWebpackPlugin({
            filename: 'editor/index.html',
            template: './src/demo/templates/template.html',
            chunks: ['demo/styles', 'themes/stencila/styles', 'demo/app.tsx'],
            templateParameters: {
              ASSET_PATH,
            },
          }),
          new HtmlWebpackPlugin({
            filename: 'index.html',
            template: './src/demo/templates/gallery.ejs',
            chunks: [
              'demo/styles',
              'demo/gallery.tsx',
              'themes/galleria/styles',
            ],
          }),
        ]
      : []

  return {
    entry,
    resolve: {
      extensions: ['.ts', '.tsx', '.js', '.css', '.html'],
      fallback: {
        path: false,
      },
    },
    mode: mode || 'development',
    target: target === 'lib' ? 'node' : 'web',
    output: {
      assetModuleFilename: fileLoaderOutputPath,
      path: path.resolve(__dirname, contentBase),
      publicPath: ASSET_PATH,
      library: 'thema',
      libraryTarget: 'umd',
      umdNamedDefine: true,
    },
    node: {
      __dirname: false,
    },
    devServer: {
      contentBase: path.join(__dirname, contentBase),
      overlay: true,
    },
    plugins: [
      new DefinePlugin({
        'process.env.ASSET_PATH': JSON.stringify(ASSET_PATH),
        'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV),
        'process.env.npm_package_version': JSON.stringify(
          process.env.npm_package_version
        ),
      }),
      new MiniCssExtractPlugin(),
      ...docsPlugins,
    ],
    module: {
      rules: [
        {
          test: /\.ts(x?)$/,
          use: {
            loader: 'ts-loader',
            options: {
              configFile: `tsconfig.${
                target === 'lib' ? 'lib' : 'browser'
              }.json`,
              compilerOptions: {
                incremental: true,
              },
            },
          },
        },
        {
          test: /\.ejs$/,
          loader: 'ejs-loader',
          options: {
            // @see https://github.com/difelice/ejs-loader/pull/45
            esModule: false,
          },
        },
        {
          test: /\.html$/i,
          // Don't transform HtmlWebpackPlugin generated file
          exclude: /template\.html$/i,
          type: 'asset/resource',
        },
        {
          test: /\.(css)$/,
          use: [
            {
              loader: MiniCssExtractPlugin.loader,
              options: {
                publicPath: (resourcePath, context) => {
                  // Make asset paths relative
                  // @see https://github.com/webpack-contrib/mini-css-extract-plugin#the-publicpath-option-as-function
                  return (
                    path.relative(
                      path.dirname(resourcePath),
                      path.join(context, 'src')
                    ) + '/'
                  )
                },
              },
            },
            {
              loader: 'css-loader',
              options: { importLoaders: 1, url: true, import: true },
            },
            'postcss-loader',
          ],
        },
        {
          test: /\.(eot|woff|woff2|svg|ttf|jpe?g|png|gif)$|html\.media\/.*$/,
          type: 'asset/resource',
        },
      ],
    },
  }
}
