# 📦 Stencila buildpack for Node.js

## Detection

Matches against a project that has:

  - a `.tool-versions` file with a `nodejs` or `node` entry, or

  - any of `package.json`, `package-lock.json`, `.nvmrc`, `main.js`, or `index.js` in its root folder

## Node.js version

The version of Node.js to be installed is determined by (in descending order of precedence):

  - the `nodejs` or `node` entry of any `.tool-versions` file,

  - the content of any `.nvmrc` file,

  - the `engines.node` property of any `package.json`, or else

  - the latest version of Node.js.

## NPM packages

NPM packages are installed into a `node_modules` folder (usually `.venv`) within the project folder. Which NPM packages and their versions to install is determined by (in descending order of precedence):

  - if a `package.json` or `package-lock.json` file is present, then `npm install` will be used to install the version of packages specified in those files (see the NPM [docs](https://docs.npmjs.com/cli/v8/commands/npm-install) for more on the exact behavior).
