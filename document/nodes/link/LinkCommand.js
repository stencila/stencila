'use strict';

import AnnotationCommand from 'substance/ui/AnnotationCommand'

/**
 * A link annotation command used instead of
 * `substance/packages/link/LinkCommand` which modifies
 * some annotation behaviour
 *
 * @class      LinkCommand (name)
 */
function LinkCommand () {
  LinkCommand.super.apply(this, arguments);
}

LinkCommand.Prototype = function () {
};

AnnotationCommand.extend(LinkCommand);

module.exports = LinkCommand;
