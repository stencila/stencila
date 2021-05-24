import * as schema from '@stencila/schema'
import { For } from 'solid-js'
import { BlockContentArray } from './BlockContent'
import { ContentArray } from './Content'

export function Table(props: { node: schema.Table }) {
  return (
    <table itemtype="https://schema.org/Table" itemscope id={props.node.id}>
      {props.node.label && <label data-itemprop="label">{props.node.label}</label>}
      {props.node.caption && (
        <figcaption>
          {typeof props.node.caption === 'string' ? (
            props.node.caption
          ) : (
            <BlockContentArray nodes={props.node.caption}></BlockContentArray>
          )}
        </figcaption>
      )}
      <tbody>
        <For each={props.node.rows}>
          {(row) => (
            <tr>
              <For each={row.cells}>
                {(cell) => (
                  <td>
                    {
                      <ContentArray
                        nodes={
                          cell.content as (
                            | schema.InlineContent
                            | schema.BlockContent
                          )[]
                        }
                      ></ContentArray>
                    }
                  </td>
                )}
              </For>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  )
}
