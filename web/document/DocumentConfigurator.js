'use strict';

var Configurator = require('substance/util/Configurator');

/**
 * A "configurator" for a document.
 * 
 * Uses the Substance package mechanism to reduce repetition.
 * See `substance/util/AbstractConfigurator` for inherited methods
 * used by `DocumentHTMLImporter`, `DocumentEditor` etc
 *
 * @class      DocumentConfigurator (name)
 */
function DocumentConfigurator() {
  DocumentConfigurator.super.apply(this, arguments);

  // Define the schema (used by importer)
  this.defineSchema({
    name: 'stencila-document',
    defaultTextType: 'paragraph'
  });

  // Import Substance packages, in alphabetical order
  this.import(require('substance/packages/base/BasePackage'));
  this.import(require('substance/packages/blockquote/BlockquotePackage'));
  this.import(require('substance/packages/code/CodePackage'));
  this.import(require('substance/packages/codeblock/CodeblockPackage'));
  this.import(require('substance/packages/emphasis/EmphasisPackage'));
  this.import(require('substance/packages/heading/HeadingPackage'));
  this.import(require('substance/packages/image/ImagePackage'));
  this.import(require('substance/packages/link/LinkPackage'));
  this.import(require('substance/packages/list/ListPackage'),{
    enableMacro: true
  });
  this.import(require('substance/packages/paragraph/ParagraphPackage'));
  this.import(require('substance/packages/strong/StrongPackage'));
  this.import(require('substance/packages/subscript/SubscriptPackage'));
  this.import(require('substance/packages/superscript/SuperscriptPackage'));
  this.import(require('substance/packages/table/TablePackage'));


  // Stencila annotation nodes

  // Link (overrides of Substance command and tool)
  this.addCommand('link', require('../../nodes/link/LinkCommand'), {nodeType: 'link'});
  this.addTool('link', require('../../nodes/link/LinkTool'));


  // Import editor packages
  this.import(require('./editors/visual/VisualEditorPackage'));

}

DocumentConfigurator.Prototype = function() {

  /**
   * Gets the file client
   * 
   * Method required by `AbstractEditor._initialize`
   *
   * @return     {<type>}  The file client.
   */
  this.getFileClient = function() {
    return null;
  };

  /**
   * Gets the save handler.
   *
   * Method required by `AbstractEditor._initialize`
   * 
   * @return     {<type>}  The save handler.
   */
  this.getSaveHandler = function() {
    return null;
  };

};

Configurator.extend(DocumentConfigurator);

module.exports = DocumentConfigurator;
