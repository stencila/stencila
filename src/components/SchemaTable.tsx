import { Table } from 'rbx'
import * as React from 'react'

export interface SchemaTableProps {
  schema: {
    [key: string]: unknown
  }
}

export const SchemaTable = ({ schema }: SchemaTableProps) => (
  <Table
    bordered={true}
    fullwidth={true}
    hoverable={true}
    narrow={true}
    striped={true}
  >
    <Table.Head>
      <Table.Row>
        <Table.Heading>Property</Table.Heading>
        <Table.Heading>Type</Table.Heading>
        <Table.Heading>Description</Table.Heading>
      </Table.Row>
    </Table.Head>

    <Table.Body>
      {Object.entries(schema).map(([key, value]) => (
        <Table.Row key={key}>
          <Table.Cell>
            <code>{key}</code>
          </Table.Cell>
          <Table.Cell>
            <code>{value.type}</code>
          </Table.Cell>
          <Table.Cell>{value.description}</Table.Cell>
        </Table.Row>
      ))}
    </Table.Body>
  </Table>
)
