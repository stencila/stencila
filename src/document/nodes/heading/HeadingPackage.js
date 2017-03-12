import { HeadingPackage, Tool } from 'substance'
import HeadingMacro from './HeadingMacro'
import InsertHeadingCommand from './InsertHeadingCommand'

export default {
  name: 'heading',
  configure: function (config) {
    config.import(HeadingPackage)
    config.addMacro(new HeadingMacro())
    config.addCommand('insert-heading-1', InsertHeadingCommand, { level: 1 })
    config.addTool('insert-heading-1', Tool, { toolGroup: 'insert' })
    config.addLabel('insert-heading-1', 'Insert Heading Level 1')
    config.addIcon('insert-heading-1', { 'text': 'H1' })
    config.addCommand('insert-heading-2', InsertHeadingCommand, { level: 2 })
    config.addTool('insert-heading-2', Tool, { toolGroup: 'insert' })
    config.addLabel('insert-heading-2', 'Insert Heading Level 2')
    config.addIcon('insert-heading-2', { 'text': 'H2' })
    config.addCommand('insert-heading-3', InsertHeadingCommand, { level: 3 })
    config.addTool('insert-heading-3', Tool, { toolGroup: 'insert' })
    config.addLabel('insert-heading-3', 'Insert Heading Level 3')
    config.addIcon('insert-heading-3', { 'text': 'H3' })

  }
}
