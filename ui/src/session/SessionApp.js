import Component from 'substance/ui/Component'

import Session from './Session'
import SessionClient from './SessionClient'

import Execution from './nodes/execution/Execution'
import ExecutionComponent from './nodes/execution/ExecutionComponent'

class SessionApp extends Component {

  constructor (...args) {
    super(...args)

    this.handleActions({
      'execute': this.execute,
      'back': this.back,
      'forward': this.forward
    })
  }

  getInitialState () {
    let data = this.props.data
    let session = new Session()
    let client = new SessionClient(data ? data.url : this.props.url)
    let execution = new Execution(session, {
      'id': 'current-execution'
    })
    return {
      execution: execution,
      session: session,
      client: client
    }
  }

  render ($$) {
    let data = this.props.data

    let el = $$('div').addClass('sc-session-app ui container')

    el.append(
      $$('h1').append(
        $$('i').addClass('ui icon terminal'),
        $$('span').text(data.short || data.address)
      ),
      $$(ExecutionComponent, {
        node: this.state.execution
      }).ref('current')
    )

    return el
  }

  execute () {
    let current = this.refs['current']
    let code = current.refs['code'].editor.getValue()

    this.state.client.execute(code, {
      input: {
        type: current.refs['inputType'].val(),
        format: current.refs['inputFormat'].val(),
        value: current.refs['inputValue'].val()
      }
    }).then(result => {
      this.state.execution.result = result
      this.rerender()
    })
  }

  back () {
    console.warn('TODO: back')
  }

  forward () {
    console.warn('TODO: forward')
  }

  refresh () {
    this.state.client.call('list').then(list => {
      this.extendState({
        list: list
      })
    })
  }

}

export default SessionApp
