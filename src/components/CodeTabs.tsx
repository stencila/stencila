import { Tab } from 'rbx'
import * as React from 'react'
import { useState } from 'react'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { atomDark } from 'react-syntax-highlighter/dist/esm/styles/prism'

interface Props {
  title: string
  data: {
    edges: {
      node: {
        relativePath: string
      }
    }[]
  }
}

export const CodeTabs = ({ title, data }: Props) => {
  const files: Array<string> = [
    `${title}.schema.yaml`,
    ...data.edges.map(({ node }) => node.relativePath)
  ]

  const schemas = files.map(file => {
    try {
      return require('../../schema/' + file)
    } catch {
      try {
        return require('../../examples/' + file)
      } catch {}
    }
  })

  const [schemaIndex, changeSchema] = useState(0)

  return (
    <div>
      <Tab.Group>
        {files.map((file, index) => (
          <Tab
            key={file}
            active={schemaIndex === index}
            onClick={() => changeSchema(index)}
          >
            {file.split('/').reverse()[0]}
          </Tab>
        ))}
      </Tab.Group>

      <SyntaxHighlighter
        className="is-marginless"
        language="json"
        style={atomDark}
      >
        {JSON.stringify(schemas[schemaIndex], null, 2)}
      </SyntaxHighlighter>
    </div>
  )
}
