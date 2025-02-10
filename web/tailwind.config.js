/**
 * This config is currently only used for themes.
 * It is applied by Parcel when building the theme CSS files.
 *
 * For configuration of Tailwind in web components see
 * `src/twind.ts`.
 */
export default {
  content: ['./src/**/*.ts'],
  safelist: ['not-prose'],
  plugins: [
    require('@tailwindcss/typography'),
    require('@tailwindcss/container-queries'),
  ],
}
