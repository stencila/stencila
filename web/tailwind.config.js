/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/*.html"],
  theme: {
    extend: {},
  },
  plugins: [require("@tailwindcss/typography")],
};
