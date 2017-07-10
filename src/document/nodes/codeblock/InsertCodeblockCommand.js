import { InsertNodeCommand } from 'substance'

export default class InsertCodeblockCommand extends InsertNodeCommand {

  createNodeData() {
    return { type: 'codeblock' }
  }

}
