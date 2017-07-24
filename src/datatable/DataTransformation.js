import { DocumentNode } from 'substance'

export default class DataTransformation extends DocumentNode {}

DataTransformation.type = 'data-transformation'

DataTransformation.schema = {
  source: { type: 'string', default: '' },
  transforms: { type: ['array', 'transform'], default: [] }
}