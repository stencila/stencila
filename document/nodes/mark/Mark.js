'use strict'

import PropertyAnnotation from 'substance/model/PropertyAnnotation'

/**
 * <span data-mark="discuss-1">...</span>
 *
 * <div data-discuss="open" id="discuss-1">
 *    <div data-comment="@peter at 2016-08-23T13:21:09Z">
 *    </div>
 * </div>
 *
 * @class      Mark (name)
 */
function Mark () {
  Mark.super.apply(this, arguments)
}

Mark.Prototype = function () {
}

PropertyAnnotation.extend(Mark)

Mark.define({
  type: 'mark',
  target: { type: 'string', default: '' }
})

module.exports = Mark
