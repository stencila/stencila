module.exports = {
  modules: false,
  plugins: [
    require('postcss-import'),
    require('postcss-import-url')({ modernBrowser: true }),
    require('postcss-url')({
      url: 'rebase'
    }),
    require('postcss-mixins')({ mixinsDir: 'src/designa/mixins' }),
    require('postcss-custom-selectors')({ importFrom: 'src/selectors.css' }),
    require('postcss-custom-media'),
    require('postcss-custom-properties')({ preserve: true }),
    require('postcss-nested'),
    require('autoprefixer'),
    require('postcss-extend'),
    require('cssnano')({ preset: 'default' }),
    require('postcss-combine-media-query'),
    require('postcss-sort-media-queries')
  ]
}
