import Thing from './types/Thing'
import * as types from './types'

// The was `const pkg = require('../package')` but since that
// doesn't work in the browser this is a temporrary workaround
const pkg = {
  name: '@stencila/schema',
  version: '0.0.0',
  homepage: 'https://stencila.github.io/schema/'
}

/**
 * A manifest describing the capabilites of a processor.
 */
export interface ProcessorManifest {
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

export default class Processor {
  
  /**
   * Get the manifest for this processor
   */
  manifest (): ProcessorManifest {
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

  /**
   * Import a `Thing`.
   *
   * @param thing The thing to be imported
   * @param format The current format of the thing as a MIME type e.g. `text/markdown`
   * @returns An instance of a class derived from `Thing`
   */
  import (thing: string | object | Thing, format: string= 'application/ld+json'): Thing {
    if (thing instanceof Thing) {
      return thing
    } else if (typeof thing === 'object') {
      return this.importObject(thing)
    } else {
      switch (format) {
        case 'application/ld+json':
          return this.importJsonLd(thing)
        default:
          throw Error(`Unhandled import format: ${format}`)
      }
    }
  }

  /**
   * Import an `Object` to a `Thing`
   *
   * This function demarshalls a plain JavaScript object into an
   * instance of a class derived from `Thing` based on the `type`
   * property of the object.
   *
   * @param object A plain JavaScript object with a `type` property
   * @returns An instance of a class derived from `Thing`
   */
  importObject (object: any): Thing {
    const type = object.type
    if (!type) throw new Error('Object is missing required "type" property')
    // @ts-ignore
    const Type = types[type]
    if (!Type) throw new Error(`Unknown type "${type}"`)
    return new Type(object)
  }

  /**
   * Import a JSON-LD document to a `Thing`
   *
   * @param jsonld A JSON-LD document with a `type` property
   * @returns An instance of a class derived from `Thing`
   */
  importJsonLd (jsonld: string): Thing {
    const object = JSON.parse(jsonld)
    return this.importObject(object)
  }

  /**
   * Export a `Thing`.
   *
   * @param thing The thing to be exported
   * @param format The format, as a MIME type, to export to e.g. `text/html`
   */
  export (thing: string | object | Thing, format: string= 'application/ld+json'): string {
    if (!(thing instanceof Thing)) thing = this.import(thing)

    switch (format) {
      case 'application/ld+json':
        return this.exportJsonLd(thing as Thing)
      default:
        throw Error(`Unhandled export format: ${format}`)
    }
  }

  /**
   * Export a `Thing` to a JSON-LD string
   *
   * @param thing The thing to be exported
   */
  exportJsonLd (thing: Thing): string {
    const obj = Object.assign({
      '@context': 'https://stencila.github.io/schema/context.jsonld'
    }, this.exportObject(thing))
    return JSON.stringify(obj)
  }

  /**
   * Export a `Thing` to an `Object`
   *
   * This function marshalls a `Thing` to a plain JavaScript object
   * having a `type` and other properties of the type of thing.
   *
   * @param thing The thing to be exported
   */
  exportObject (thing: Thing): {[key: string]: any} {
    const obj: {[key: string]: any} = {}
    obj['type'] = thing.type

    for (let [key, value] of Object.entries(thing)) {
      if (typeof value === 'string' && value.length === 0) continue
      if (Array.isArray(value) && value.length === 0) continue

      if (Array.isArray(value)) {
        obj[key] = value.map(item => (item instanceof Thing) ? this.exportObject(item) : item)
      } else if (value instanceof Thing) {
        obj[key] = this.exportObject(value)
      } else {
        obj[key] = value
      }
    }

    return obj
  }

  /**
   * Convert a thing from one format to another.
   *
   * @param thing The thing to convert as a string
   * @param from The current format of the thing as a MIME type e.g. `text/markdown`
   * @param to The desired format for the thing as a MIME type e.g. `text/html`
   */
  convert (thing: string, from: string= 'application/ld+json', to: string= 'application/ld+json'): string {
    return this.export(this.import(thing, from), to)
  }

  /**
   * Compile a thing
   *
   * @param thing The thing to compile
   * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
   */
  compile (thing: string | object | Thing, format: string = 'application/ld+json'): Thing {
    thing = this.import(thing, format)
    return thing as Thing
  }

  /**
   * Build a `Thing`.
   *
   * The `build` function, like the `compile` function is used to prepare a thing
   * for execution. However, it usually involves the creation of build artifacts
   * (which may take some time to build) that are exernal to the thing
   * e.g. a binary executable or Docker image.
   * Like `compile`, it may add or modify properties of the thing
   * such as providing a URL to the built artifacts.
   *
   * @param thing The thing to build
   * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
   */
  build (thing: string | object | Thing, format: string = 'application/ld+json'): Thing {
    thing = this.compile(thing, format)
    return thing as Thing
  }

  /**
   * Execute a thing
   *
   * @param thing The thing to execute
   * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
   */
  execute (thing: string | object | Thing, format: string= 'application/ld+json'): Thing {
    thing = this.build(thing, format)
    return thing as Thing
  }
}
