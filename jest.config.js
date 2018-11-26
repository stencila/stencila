module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: [
    "**/tests/**/*.test.{ts,js}"
  ],
  collectCoverageFrom : [
    "src/**/*.{ts,js}",
    "!src/**/index.ts"
  ]
};