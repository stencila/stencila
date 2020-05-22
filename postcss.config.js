module.exports = {
  modules: false,
  plugins: [
    require('postcss-import'),
    require('postcss-import-url')({ modernBrowser: true }),
    require('postcss-url')({
      url: 'rebase',
    }),
    require('postcss-custom-selectors')({
      importFrom: ['src/selectors.css', 'src/extensions/code/styles.css'],
    }),
    require('postcss-custom-media'),
    require('postcss-custom-properties')({ preserve: true }),
    require('postcss-nested'),
    require('postcss-nested-import'),
    // Many browsers don’t support compound `:not()` selectors, this splits it
    require('postcss-selector-not'),
    // We remove the PrismJS specific modifier when used in `:not()` selectors
    // see ./src/scripts/selectors.ts:61
    require('postcss-selector-replace')({
      before: [/\[class\*=language-\]\)/gm],
      after: [')'],
    }),
    require('autoprefixer'),
    require('postcss-extend'),
    require('postcss-mixins'),
    require('cssnano')({ preset: 'default' }),
    require('postcss-combine-media-query'),
    require('postcss-sort-media-queries'),
  ],
}
