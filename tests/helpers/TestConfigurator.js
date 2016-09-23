import Configurator from 'substance/util/Configurator'

/**
 * A "configurator" for loading individual packages during testing.
 *
 * @class      TestConfigurator (name)
 */
class TestConfigurator extends Configurator {

  constructor (packages) {
    super()
    packages.forEach((packag) => {
      this.import(packag)
    })
  }

}

export default TestConfigurator
