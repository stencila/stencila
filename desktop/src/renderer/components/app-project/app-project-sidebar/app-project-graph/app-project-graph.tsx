import { Component, h, Host, Prop, State } from '@stencil/core'
import { Graph, GraphEvent } from 'stencila'
import { CHANNEL } from '../../../../../preload/channels'
import { captureError } from '../../../../../preload/errors'
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

  private updateGraph = (e: GraphEvent) => {
    this.graph = e.graph
  }

  private subscribeToUpdates = () => {
    window.api.receive(CHANNEL.PROJECTS_GRAPH, (e) =>
      this.updateGraph(e as GraphEvent)
    )
  }

  private unsubscribeFromUpdates = () => {
    window.api.removeAll(CHANNEL.PROJECTS_GRAPH)
    client.projects
      .unsubscribe(this.projectPath, ['graph'])
      .catch((err) => captureError(err))
  }

  componentWillLoad() {
    return client.projects.graph(this.projectPath).then((res) => {
      this.subscribeToUpdates()

      try {
        this.graph = JSON.parse(res.value) as Graph
      } catch (err) {
        errorToast(err)
      }
    })
  }

  disconnectedCallback() {
    this.unsubscribeFromUpdates()
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
