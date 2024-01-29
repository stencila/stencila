import { defineConfig } from '@twind/core'
import presetAutoprefix from '@twind/preset-autoprefix'
import presetTailwind from '@twind/preset-tailwind/base'
import install from '@twind/with-web-components'
/**
 * The configuration for `twind` Tailwind-in-JS
 *
 * This configuration only applies to the use of Tailwind
 * within TypeScript/Javascript (i.e. where `installTwind` is
 * called).
 *
 * For configuration of Tailwind for themes see the `tailwind.config.js` file.
 */
export const config = defineConfig({
  presets: [presetAutoprefix(), presetTailwind()],

  theme: {
    extend: {
      fontFamily: {
        sans: ['Lato', 'Montserrat'],
      },
      colors: {
        brand: {
          blue: '#2568ef',
          green: '#66ff66',
          red: '#e53e3e',
          yellow: '#ecc94b',
        },
        gray: {
          200: '#dedede',
          'wild-sand': '#f5f5f5',
          shady: '#9d9d9d',
          aluminium: '#999999',
          'mine-shaft': '#333333',
        },
        black: '#171817',
        green: {
          '000': '#f5fff5',
        },
      },
    },
  },

  hash: false,
})

export const withTwind = () => install(config)
