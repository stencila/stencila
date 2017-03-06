import { InsertNodeCommand, uuid } from 'substance'

class InsertCellCommand extends InsertNodeCommand {

  createNodeData() {

    return {
      id: uuid('cell'),
      type: 'cell',
      expression: '',
    }

  }

}

export default InsertCellCommand
