'use strict';

import HTMLImporter from 'substance/model/HTMLImporter'

import DocumentModel from './DocumentModel'
import DefaultHTMLConverter from './nodes/default/DefaultHTMLConverter'

/**
 * Imports HTML into a Stencila Document
 *
 * @class      HTMLImporter (name)
 */
function DocumentHTMLImporter (options) {
  DocumentHTMLImporter.super.call(this, {
    // Required configuration for an importer
    DocumentClass: DocumentModel,
    schema: options.configurator.getSchema(),
    converters: options.configurator.getConverterRegistry().get('html')
  });
}

DocumentHTMLImporter.Prototype = function () {
  /**
   * Convert HTML into a Stencila Document
   *
   * This method must be provided when extending HTMLImporter
   */
  this.convertDocument = function (els) {
    // The `containerId` argument should have the same
    // value as the `containerId` used by `ContainerEditor`
    this.convertContainer(els, 'content');
  };

  /**
   * Method override to provide a default for
   * importing HTML elements not matched by `converters`
   */
  this.defaultConverter = function (el, converter) {
    var nodeData = DefaultHTMLConverter.createNodeData();
    DefaultHTMLConverter.import(el, nodeData, converter);
    return nodeData;
  };
};

HTMLImporter.extend(DocumentHTMLImporter);

module.exports = DocumentHTMLImporter;
