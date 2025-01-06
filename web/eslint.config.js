import js from '@eslint/js'
import importPlugin from 'eslint-plugin-import'
import litPlugin from 'eslint-plugin-lit'
import wcPlugin from 'eslint-plugin-wc'
import tseslint from 'typescript-eslint'

export default [
  ...tseslint.config(
    js.configs.recommended,
    tseslint.configs.recommended,
    importPlugin.flatConfigs.recommended,
    importPlugin.flatConfigs.typescript,
    wcPlugin.configs['flat/recommended'],
    litPlugin.configs['flat/recommended']
  ),
  {
    languageOptions: {
      globals: {
        browser: true,
        es2021: true,
        es6: true,
        node: true,
      },
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
      },
    },
    settings: {
      wc: {
        elementBaseClasses: ['LitElement'], // Recognize `LitElement` as a Custom Element base class
      },
    },
    rules: {
      semi: ['error', 'never'],
      quotes: [
        'error',
        'single',
        { avoidEscape: true, allowTemplateLiterals: true },
      ],
      '@typescript-eslint/ban-types': 'off',
      // Emulate the TypeScript style of exempting names starting with _
      // https://typescript-eslint.io/rules/no-unused-vars/#benefits-over-typescript
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          caughtErrors: 'all',
          caughtErrorsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
      'import/order': [
        'error',
        {
          groups: [
            'builtin',
            'external',
            'internal',
            'parent',
            'sibling',
            'index',
          ],
          alphabetize: { order: 'asc' },
          'newlines-between': 'always',
        },
      ],
    },
    ignores: ['.prettierrc'],
  },
]
