import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default [
  ...tseslint.config(js.configs.recommended, tseslint.configs.recommended),
  {
    languageOptions: {
      globals: {
        browser: true,
        es2021: true,
        node: true,
      },
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
      },
    },
    rules: {
      "@typescript-eslint/ban-types": "off",
    },
  },
];
