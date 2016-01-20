'use strict';

var extend = require('lodash/object/extend');
var capitalize = require('lodash/string/capitalize');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var ControllerTool = require('substance/ui/ControllerTool');

function CommitMessagePrompt() {
  Component.apply(this, arguments);
}

CommitMessagePrompt.Prototype = function() {

  this.onSave = function(e) {
    e.preventDefault();
    this.props.tool.performSave();
  };

  this.onDelete = function(e) {
    e.preventDefault();
    this.props.tool.deleteLink();
  };

  this.render = function() {
    var el = $$('div').addClass('se-prompt');

    el.append([
      $$('div').addClass('se-prompt-title').append('Save changes'),
      $$('input').attr({type: 'text', placeholder: 'Enter commit message', value: ''})
                 .ref('url')
                 .htmlProp('autofocus', true),
                 // .on('change', this.onSave),
      $$('a').attr({href: '#'})
             .addClass('se-save-btn')
             .append(this.i18n.t('save'))
             .on('click', this.onSave)
    ]);
    return el;
  };
};

Component.extend(CommitMessagePrompt);

function SaveTool() {
  ControllerTool.apply(this, arguments);

  var ctrl = this.getController();
  ctrl.connect(this, {
    'command:executed': this.onCommandExecuted
  });
}

SaveTool.Prototype = function() {

  this.dispose = function() {
    var ctrl = this.getController();
    ctrl.disconnect(this);
  };

  this.onCommandExecuted = function(info, commandName) {
    if (commandName === this.constructor.static.command) {
      // Toggle the edit prompt when either edit is requested or a new link has been created
      if (info.status === 'save-requested') {
        this.togglePrompt();
      }
    }
  };

  this.togglePrompt = function() {
    var newState = extend({}, this.state, {showPrompt: !this.state.showPrompt});
    this.setState(newState);
  };

  this.performSave = function() {
    var ctrl = this.getController();
    ctrl.saveDocument();
    this.extendState({showPrompt: false});
  };

  this.getLink = function() {
    return this.getDocument().get(this.state.annotationId);
  };

  this.render = function() {
    var title = this.props.title || capitalize(this.getName());

    if (this.state.mode) {
      title = [capitalize(this.state.mode), title].join(' ');
    }

    var el = $$('div')
      .addClass('sc-save-tool se-tool')
      .attr('title', title);

    if (this.state.disabled) {
      el.addClass('sm-disabled');
    }

    var button = $$('button').on('click', this.onClick);

    button.append(this.props.children);
    el.append(button);

    // When we are in edit mode showing the edit prompt
    if (this.state.showPrompt) {
      el.addClass('sm-active');
      var prompt = $$(CommitMessagePrompt, {tool: this});
      el.append(prompt);
    }
    return el;
  };
};

ControllerTool.extend(SaveTool);

SaveTool.static.name = 'save';
SaveTool.static.command = 'save';

module.exports = SaveTool;
