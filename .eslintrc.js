module.exports = {
  "extends": "standard",
  "parserOptions": {
      "ecmaVersion": 6,
      "sourceType": "module"
  },
  "env": {
    "browser": true,
    "node": true,
    "es6": true
  },
  "rules": {
    "indent": ["error", 2],
    "valid-jsdoc": ["error", {
      "requireReturn": false
    }]
  }
}
