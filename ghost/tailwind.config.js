/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./*.hbs",
    "./**/*.hbs",
  ],
  theme: {
    extend: {},
  },
  plugins: [require('autoprefixer')],
}

