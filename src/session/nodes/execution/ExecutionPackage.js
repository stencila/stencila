import Execution from './Execution'
import ExecutionComponent from './ExecutionComponent'

export default {
  name: 'execution',
  configure: function (config) {
    config.addNode(Execution)
    config.addComponent('execution', ExecutionComponent)
  }
}
