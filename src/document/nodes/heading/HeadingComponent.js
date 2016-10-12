import HeadingComponentBase from 'substance/packages/heading/HeadingComponent'

/**
 * A component for `Heading` nodes
 *
 * Extends Substance `HeadingComponent` but add an event to rerender
 * on a change to the level
 *
 * @class      HeadingComponent (name)
 */
class HeadingComponent extends HeadingComponentBase {
  didMount () {
    this.props.node.on('level:changed', this.rerender, this)
  }

  dispose () {
    this.props.node.off(this)
  }
}

export default HeadingComponent
