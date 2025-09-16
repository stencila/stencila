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

      // Formatting rules

      semi: ['error', 'never'],
      quotes: [
        'error',
        'single',
        { avoidEscape: true, allowTemplateLiterals: true },
      ],

      // Spacing & Indentation
      'no-trailing-spaces': 'error',
      'no-multiple-empty-lines': ['error', { max: 1, maxEOF: 0, maxBOF: 0 }],
      'space-before-blocks': 'error',
      'keyword-spacing': ['error', { before: true, after: true }],
      'space-infix-ops': 'error',
      'object-curly-spacing': ['error', 'always'],
      'array-bracket-spacing': ['error', 'never'],
      'computed-property-spacing': ['error', 'never'],
      'space-in-parens': ['error', 'never'],
      'space-before-function-paren': ['error', {
        anonymous: 'always',
        named: 'never',
        asyncArrow: 'always'
      }],

      // Line Length & Wrapping
      'max-len': ['error', {
        code: 120,
        ignoreUrls: true,
        ignoreStrings: true,
        ignoreTemplateLiterals: true,
        ignoreRegExpLiterals: true
      }],
      'array-bracket-newline': ['error', 'consistent'],
      'object-curly-newline': ['error', {
        ObjectExpression: { consistent: true },
        ObjectPattern: { consistent: true },
        ImportDeclaration: { consistent: true },
        ExportDeclaration: { consistent: true }
      }],

      // Punctuation & Syntax
      'comma-spacing': ['error', { before: false, after: true }],
      'comma-style': ['error', 'last'],
      'arrow-parens': ['error', 'always'],
      'arrow-spacing': ['error', { before: true, after: true }],
      'brace-style': ['error', '1tbs', { allowSingleLine: true }],
      'dot-location': ['error', 'property'],

      // Other Formatting
      'jsx-quotes': ['error', 'prefer-double'],
      'template-curly-spacing': ['error', 'never'],
      'no-mixed-spaces-and-tabs': 'error',
      'eol-last': ['error', 'always'],
      'no-whitespace-before-property': 'error',
      'padded-blocks': ['error', 'never'],
      'block-spacing': ['error', 'always'],
      'key-spacing': ['error', { beforeColon: false, afterColon: true }],
      'switch-colon-spacing': ['error', { after: true, before: false }],
    },
  },
  // Ignore patterns
  {
    ignores: [
      '.prettierrc',
      '**/.next/**',
      '**/*.config.js',
      '**/*.config.mjs',
      '**/*.config.ts',
      '**/*.d.ts',
      '**/build/**',
      '**/coverage/**',
      '**/dist/**',
      '**/node_modules/**',
      '**/out/**',
      '**/public/**',
    ],
  },
  // CommonJS files configuration
  {
    files: ['**/*.cjs'],
    languageOptions: {
      sourceType: 'commonjs',
      globals: {
        __dirname: 'readonly',
        __filename: 'readonly',
        require: 'readonly',
        module: 'readonly',
        exports: 'readonly',
        process: 'readonly',
        console: 'readonly',
        Buffer: 'readonly',
      },
    },
    rules: {
      '@typescript-eslint/no-require-imports': 'off',
      '@typescript-eslint/no-var-requires': 'off',
    },
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
      'import/no-unresolved': ['error', { ignore: ['vscode'] }],
    },
  },
]