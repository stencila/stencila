import js from '@eslint/js'
import importPlugin from 'eslint-plugin-import'
import litPlugin from 'eslint-plugin-lit'
import wcPlugin from 'eslint-plugin-wc'
import tseslint from 'typescript-eslint'

export default [
  // Base configurations
  ...tseslint.config(
    js.configs.recommended,
    tseslint.configs.recommended,
    importPlugin.flatConfigs.recommended,
    importPlugin.flatConfigs.typescript
  ),
  // Global configuration for all workspaces
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
  },
  // Ignore patterns
  {
    ignores: [
      '.prettierrc',
      'out',
      'dist',
      '**/*.d.ts',
      '**/node_modules/**',
      '**/coverage/**',
      '**/build/**',
      '**/.next/**',
      '**/public/**',
      '**/*.config.js',
      '**/*.config.mjs',
      '**/*.config.ts',
    ],
  },
  // Web component rules for web workspace
  {
    files: ['web/**/*.ts', 'web/**/*.js'],
    ...wcPlugin.configs['flat/recommended'],
    ...litPlugin.configs['flat/recommended'],
    settings: {
      wc: {
        elementBaseClasses: ['LitElement'], // Recognize `LitElement` as a Custom Element base class
      },
    },
  },
  // Rules for vscode workspace
  {
    files: ['vscode/**/*.ts', 'vscode/**/*.js'],
    rules: {
      '@typescript-eslint/naming-convention': [
        'warn',
        {
          selector: 'import',
          format: ['camelCase', 'PascalCase'],
        },
      ],
      curly: 'warn',
      eqeqeq: 'warn',
      'no-throw-literal': 'warn',
    },
  },
]