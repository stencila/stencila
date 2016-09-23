import Container from 'substance/model/Container'

function Discussion () {
  Discussion.super.apply(this, arguments)
}

Container.extend(Discussion)

Discussion.define({
  type: 'discussion',
  status: { type: 'string', default: 'open' },
  nodes: { type: ['id'], default: [] }
})

export default Discussion
