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
        sans: [
          'Inter',
          // The default Tailwind font stack from https://tailwindcss.com/docs/font-family
          'ui-sans-serif',
          'system-ui',
          'sans-serif',
          'Apple Color Emoji',
          'Segoe UI Emoji',
          'Segoe UI Symbol',
          'Noto Color Emoji',
        ],
        mono: ['IBM Plex Mono', 'monospace'],
      },
      fontSize: {
        '2xs': '0.625rem',
      },
      dropShadow: {
        '2xl': '0 0 0.15em rgba(37, 104, 239, 1)',
      },
      maxWidth: {
        '1/5': '20%',
      },
      colors: {
        transparent: 'transparent',
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
          '150': '#f2f2f2',
          '200': '#dedede',
          '300': '#c7c7c7',
          '400': '#b0b0b0',
          '500': '#999999',
          '600': '#808080',
          '700': '#666666',
          '800': '#4d4d4d',
          '900': '#333333',
        },
        black: '#171817',
        greene: {
          '000': '#f5fff5',
          100: '#e5ffe5',
          200: '#d9f2d9',
          900: '#0B4C46',
        },
        blue: {
          50: '#f1f5fe',
          200: '#b1c9fa',
          400: '#77a0f5',
          700: '#2568ef',
          800: '#174bb3',
          900: '#092d77',
        },
      },
    },
  },

  hash: false,
})

export const withTwind = () => install(config)

export type TwTheme = {
  [index: string]: {
    [index: string]:
      | string
      | string[]
      | {
          [index: string]: string
        }
  }
}
