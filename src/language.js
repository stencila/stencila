/**
 * @namespace language
 */

/**
 * A list of language (shortnames) that we currently
 * support
 * 
 * @type {Array}
 */
export const LIST = [
  'js', 'py', 'r', 'sql'
]

/**
 * Get the short name (code) for a language
 *
 * @param  {string} lang - Languge name
 * @return {string} - Short name of language
 */
export function shortname (lang) {
  lang = lang.toLowerCase()
  return {
    javascript: 'js',
    julia: 'jl',
    python: 'py'
  }[lang] || lang
}

/**
 * Get the language long name from a short name
 *
 * @param  {string} lang - Languge name
 * @return {string} - Long name of language
 */
export function longname (lang) {
  return {
    jl: 'Julia',
    js: 'JavaScript',
    py: 'Python',
    r: 'R',
    sql: 'SQL'
  }[shortname(lang)] || lang
}

/**
 * Get the language comment character(s)
 *
 * @param  {string} lang - Languge name
 * @return {string} - Single line comment character(s)
 */
export function comment (lang) {
  return {
    js: '//',
    sql: '--'
  }[shortname(lang)] || '#'
}
