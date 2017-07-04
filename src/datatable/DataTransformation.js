import { DocumentNode } from 'substance'

export default DataTransformation extends DocumentNode {}

DataTransformation.type = 'data-transformation'

DataTransformation.schema = {
  source: { type: 'string', default: '' },
  transforms: { type: ['array', 'transform'], default: [] }
}