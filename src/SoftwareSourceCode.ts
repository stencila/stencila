import { type, property } from './decorators'
import { Date, Number, Text, URL } from './dataTypes'
import ComputerLanguage from './ComputerLanguage'
import CreativeWork from './CreativeWork'
import Organization from './Organization'
import Person from './Person'
import SoftwareApplication from './SoftwareApplication'

@type('schema:SoftwareSourceCode')
export default class SoftwareSourceCode extends CreativeWork {
  @property('schema:codeRepository')
  codeRepository: URL = ''

  @property('schema:codeSampleType')
  codeSampleType: Text = ''

  @property('codemeta:maintainer', 'list')
  maintainers: Array<Organization | Person> = []

  @property('schema:programmingLanguage', 'list')
  programmingLanguages: Array<ComputerLanguage> = []

  @property('schema:runtimePlatform')
  runtimePlatform: Text = ''

  @property('schema:targetProduct', 'list')
  targetProducts: Array<SoftwareApplication> = []
}
