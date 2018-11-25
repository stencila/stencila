const pkg = require('../package')

/**
 * A manifest describing the capabilites of a processor.
 */
export interface Manifest {
  /**
   * A description of the Stencila package in which this
   * processor is implemented
   */
  stencila: {},

  /**
   * Services that this processor implements
   */
  services: {
    /**
     * A list of MIME types that the processor is able to import
     */
    import: Array<string>

    /**
     * A list of MIME types that the processor is able to export
     */
    export: Array<string>

    /**
     * A list of types that this processor is able to compile
     */
    compile: Array<string>

    /**
     * A list of types that this processor is able to build
     */
    build: Array<string>

    /**
     * A list of types that this processor is able to execute
     */
    execute: Array<string>
  }
}

/**
 * Get the manifest for this processor
 */
export default function manifest (): Manifest {
  return {
    stencila: {
      name: pkg.name,
      url: pkg.homepage,
      version: pkg.version
    },
    services: {
      import: ['application/ld+json'],
      export: ['application/ld+json'],
      compile: [],
      build: [],
      execute: []
    }
  }
}
