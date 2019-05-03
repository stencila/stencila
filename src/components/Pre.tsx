import * as React from 'react'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { atomDark } from 'react-syntax-highlighter/dist/esm/styles/prism'

/**
 * Markdown code blocks tagged with a language (such as ``` javascript) apply a class of
 * `language-X`, where X is the tagged language. The React-Syntax-Highlighter plugin however
 * requires the `language` prop to not have the `language-` prefix. This function matches and extracts
 * the language name, or the last class name.
 * @param {string} s Raw string of either the language name or the CSS class name containing the language.
 */
const languageRegEx = new RegExp('((?:language-?)\\w+|\\w+$)')
const stripLangPrefix = (s: string) =>
  languageRegEx.exec(s)[1].replace('language-', '')

interface StyledPre extends React.HTMLAttributes<HTMLPreElement> {
  // Children can be either strings found in JSON file contents, or code blocks found in MDX files represented as React nodes.
  children?:
    | string
    | {
        props: {
          children: React.ReactNode
          className: string
        }
      }
}

export const Pre = (props: StyledPre) => {
  const { children, ...rest } = props

  if (!children) return null

  const content =
    typeof children === 'string'
      ? { children, language: 'json' }
      : {
          children: children.props.children,
          language: stripLangPrefix(children.props.className)
        }

  return (
    <SyntaxHighlighter
      className="is-marginless"
      language={content.language}
      style={atomDark}
      {...rest}
    >
      {content.children}
    </SyntaxHighlighter>
  )
}
