import { Component, h, Host, Prop, State } from '@stencil/core'
import { Graph, GraphEvent } from 'stencila'
import { CHANNEL } from '../../../../../preload/channels'
import { client } from '../../../../client'
import { errorToast } from '../../../../utils/errors'

@Component({
  tag: 'app-project-graph',
  styleUrl: 'app-project-graph.css',
  shadow: true,
})
export class AppProjectGraph {
  @Prop() projectPath: string

  @State() graph?: Graph

  private subscribeToUpdates = () => {
    window.api.receive(CHANNEL.PROJECTS_GRAPH, (event) => {
      const e = event as GraphEvent
      console.log('new graph', e)
      this.graph = e.graph
    })
  }

  componentWillLoad() {
    return client.projects.graph(this.projectPath).then((res) => {
      this.subscribeToUpdates()

      try {
        const parsedGraph = JSON.parse(res.value)
        console.log('og graph', parsedGraph)
        this.graph = parsedGraph
      } catch (err) {
        errorToast(err)
      }
    })
  }

  render() {
    return (
      <Host>
        {/* @ts-ignore */}
        <stencila-project-graph graph={this.graph}></stencila-project-graph>
      </Host>
    )
  }
}
