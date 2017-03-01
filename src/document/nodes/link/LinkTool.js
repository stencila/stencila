import { AnnotationTool } from 'substance'

/**
 * A tool for editing `Print` nodes
 *
 * A link tool used instead of `substance/packages/link/EditLinkTool`
 * It implements both the on/off of the link annotation as well
 * as it's editing
 *
 * Updates the node `source` property on the `change` event so that
 * errors don't get generated for incomplete input
 *
 * @class      LinkTool (name)
 */
class LinkTool extends AnnotationTool {

  render ($$) {
    var node = this.props.node
    return super.render($$)
      .addClass('sc-link-tool')
      .append(
        $$('div')
          .ref('details')
          .addClass('se-details')
          .append(
            $$('input')
              .ref('url')
              .addClass('se-url')
              .attr({
                placeholder: 'URL address',
                title: 'Link URL'
              })
              .val(node ? node.url : null)
              .on('change', function (event) {
                var session = this.context.documentSession
                session.transaction(function (tx) {
                  tx.set([node.id, 'url'], event.target.value)
                })
              }),
            $$('a')
              .ref('open')
              .addClass('se-open')
              .attr({
                href: node ? node.url : null,
                title: 'Open link',
                target: '_blank'
              })
              .append(
                $$('button')
                  .append(
                    $$('i')
                      .addClass('fa fa-external-link-square')
                  )
              )
          )
      )
  }

}

export default LinkTool
