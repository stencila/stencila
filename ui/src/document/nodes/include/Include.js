import { BlockNode } from 'substance'

class Include extends BlockNode {

  refresh () {
    let host = this.document.host
    host.open(this.address).then(component => {
      // component.select(this.selector)
      component.html.then(html => {
        this.content = html
        this.emit('changed')
      })
    })
  }

}

Include.define({
  type: 'include',

  address: { type: 'string', default: '' },
  selector: { type: 'string', default: '' },
  input: { type: 'string', default: '' }
})

export default Include
