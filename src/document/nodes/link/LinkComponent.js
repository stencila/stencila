import { LinkPackage } from 'substance'

const LinkComponentBase = LinkPackage.LinkComponent

/**
 * A component for `Link` nodes
 *
 * Extends Substance `LinkComponent` but add an event to rerender
 * on a change to the `url` property
 *
 * @class      LinkComponent (name)
 */
class LinkComponent extends LinkComponentBase {

  didMount () {
    this.props.node.on('url:changed', this.rerender, this)
  }

  dispose () {
    this.props.node.off(this)
  }

}

export default LinkComponent
