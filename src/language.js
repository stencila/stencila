
/**
 * Get the short name (code) for a language
 *
 * @param  {string} lang - Languge name
 * @return {string} - Short name of language
 */
export function shortname (lang) {
  return {
    javascript: 'js',
    js: 'js',
    julia: 'jl',
    jl: 'jl',
    python: 'py',
    py: 'py',
    r: 'r',
    sql: 'sql'
  }[lang.toLowerCase()] || null
}

/**
 * Get the language long name from a short name
 *
 * @param  {string} lang - Languge name
 * @return {string} - Long name of language
 */
export function longname (lang) {
  return {
    javascript: 'JavaScript',
    js: 'JavaScript',
    julia: 'Julia',
    jl: 'Julia',
    python: 'Python',
    py: 'Python',
    r: 'R',
    sql: 'SQL'
  }[lang.toLowerCase()] || null
}
