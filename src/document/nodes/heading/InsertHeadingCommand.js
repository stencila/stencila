import { InsertNodeCommand, uuid } from 'substance'

class InsertHeadingCommand extends InsertNodeCommand {

  createNodeData() {
    return {
      id: uuid('heading'),
      type: 'heading',
      level: this.config.level || 1
    }
  }
}

export default InsertHeadingCommand
