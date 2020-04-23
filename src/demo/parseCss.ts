// A simple modification based on https://github.com/nkt/css-variables-parser
// Adds support for `:--root` selector and TypeScript definitions

import postcss, { ChildNode, Declaration, Rule } from 'postcss'
import { translate } from '../util'

const isVariableDeclaration = (decl: ChildNode): decl is Declaration => {
  return (
    decl.type === 'decl' &&
    Boolean(decl.value) === true &&
    decl.prop.startsWith('--')
  )
}

const isOutsideRootNode = (rule: Rule): boolean => {
  return !(
    rule.selectors.length === 1 ||
    // TODO: Fix `translate` function never matching selector
    rule.selectors[0] === '[itemtype~="http://schema.org/Article"]' ||
    rule.selectors[0] === ':root' ||
    rule.selectors[0] === translate(':--root') ||
    // TODO: Remove once this branch is rebased on https://github.com/stencila/thema/pull/85
    rule.selectors[0] === '[itemscope=root]' ||
    rule.selectors[0] === translate(':--Article') ||
    rule.parent.type === 'root'
  )
}

/**
 * Given a CSS stylesheet, parses it and returns an object with all top level CSS variables and their values.
 *
 * @function getCssVariables
 * @param {string} css - CSS to parse
 * @return {Record<string, string>} Key/value pairs of found variables name and their values
 */
export const getCssVariables = (css: string): Record<string, string> => {
  const root = postcss.parse(css)

  const variables: Record<string, string> = {}
  root.walkRules((rule) => {
    if (isOutsideRootNode(rule)) {
      return
    }

    rule.each((decl: ChildNode) => {
      if (isVariableDeclaration(decl)) {
        const name = decl.prop.slice(2)
        variables[name] = decl.value
      }
    })
  })

  return variables
}
