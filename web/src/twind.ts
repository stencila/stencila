import { defineConfig } from '@twind/core'
import presetAutoprefix from '@twind/preset-autoprefix'
import presetTailwind from '@twind/preset-tailwind/base'
import theme from '@twind/preset-tailwind/defaultTheme'
import presetTypography from '@twind/preset-typography/'
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
  presets: [presetAutoprefix(), presetTailwind(), presetTypography()],
  theme: {
    ...theme,
    extend: {
      fontFamily: {
        sans: ['Lato', 'Montserrat'],
      },
      dropShadow: {
        '2xl': '0 0 0.15em rgba(37, 104, 239, 1)',
      },
      colors: {
        white: '#fff',
        brand: {
          blue: '#2568ef',
          green: '#66ff66',
          red: '#e53e3e',
          yellow: '#ecc94b',
        },
        grey: {
          // TODO: replace with design spec colours
          'wild-sand': '#f5f5f5',
          shady: '#9d9d9d',
          aluminium: '#999999',
          'mine-shaft': '#333333',

          '100': '#fefefe',
          '200': '#DEDEDE',
          '300': '#C7C7C7',
          '400': '#B0B0B0',
          '500': '#999999',
          '600': '#808080',
          '700': '#666666',
          '800': '#4d4d4d',
          '900': '#333333',
        },
        black: '#171817',
        green: {
          '000': '#f5fff5',
          200: '#D9F2D9',
        },
      },
    },
  },

  hash: false,
})

export const withTwind = () => install(config)
