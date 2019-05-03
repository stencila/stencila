import { Tab } from 'rbx'
import * as React from 'react'
import { useState } from 'react'
import { Pre } from './Pre'

interface Props {
  data: {
    edges: {
      node: {
        relativePath: string
      }
    }[]
  }
}

export const CodeTabs = ({ data }: Props) => {
  const [fetchedData, setData] = useState([])
  const [schemaIndex, changeSchema] = useState(0)

  const filePaths: Array<string> = [
    ...data.edges.map(({ node }) => node.relativePath)
  ]

  const getFileContents = async (files: Array<string>) => {
    const fileContents = await files.map(file => {
      try {
        return require(`../../dist/built/${file}`)
      } catch {
        try {
          return require(`../../examples/${file}`)
        } catch {}
      }
    })

    setData(fileContents)
  }

  React.useEffect(() => {
    getFileContents(filePaths)
  }, [data.edges])

  return (
    <div>
      <Tab.Group>
        {filePaths.map((file, index) => (
          <Tab
            key={file}
            active={schemaIndex === index}
            onClick={() => changeSchema(index)}
          >
            {file
              .split('/')
              .reverse()[0]
              .replace('yaml', 'json')}
          </Tab>
        ))}
      </Tab.Group>

      <Pre>{JSON.stringify(fetchedData[schemaIndex], null, 2)}</Pre>
    </div>
  )
}
