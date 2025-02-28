/** @type {import('tailwindcss').Config} */
export default {
  mode: 'jit',
  content: [
    "./*.hbs",
    "./**/*.hbs",
    "./assets/js/**/*.js"
  ],
  theme: {
    extend: {
      maxWidth: {
        content: '96rem'
      },
      fontFamily: {
        sans: [
          'Inter',
          'ui-sans-serif',
          'system-ui',
          'sans-serif',
          'Apple Color Emoji',
          'Segoe UI Emoji',
          'Segoe UI Symbol',
          'Noto Color Emoji',
        ],
        serif: [
          '"Source Serif 4"',
          'ui-serif', 
          'Georgia', 
          'Cambria',
          '"Times New Roman"',
          'Times',
          'serif',
          'Apple Color Emoji',
          'Segoe UI Emoji',
          'Segoe UI Symbol',
          'Noto Color Emoji',
        ]
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
  plugins: [
    require('autoprefixer'),
    require('@tailwindcss/typography'),
  ],
}

