import tseslint from 'typescript-eslint'

export default [
    ...tseslint.config(
        tseslint.configs.recommended
    ),
    {
        languageOptions: {
            parserOptions: {
                ecmaVersion: 6,
                sourceType: "module"
            },
        },
        rules: {
            "@typescript-eslint/naming-convention": [
                "warn",
                {
                    "selector": "import",
                    "format": [ "camelCase", "PascalCase" ]
                }
            ],
            "curly": "warn",
            "eqeqeq": "warn",
            "no-throw-literal": "warn",
            "semi": "off"
        },
        ignores: [
            "out",
            "dist",
            "**/*.d.ts"
        ]
    }
]
