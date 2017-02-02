module.exports = {
  language: {
    code: function (lang) {
      if (!lang) return null

      let code = lang.toLowerCase()
      code = {
        javascript: 'js',
        julia: 'jl',
        python: 'py'
      }[code] || code
      return code
    }
  }
}
