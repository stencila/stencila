'use strict';

import Subscript from 'substance/packages/subscript/Subscript'
import SubscriptHTMLConverter from 'substance/packages/subscript/SubscriptHTMLConverter'
import SubscriptXMLConverter from 'substance/packages/subscript/SubscriptXMLConverter'
import AnnotationCommand from 'substance/ui/AnnotationCommand'
import AnnotationComponent from 'substance/ui/AnnotationComponent'
import AnnotationTool from 'substance/ui/AnnotationTool'

export default {
  name: 'subscript',
  configure: function(config) {
    config.addNode(Subscript);
    config.addConverter('html', SubscriptHTMLConverter);
    config.addConverter('xml', SubscriptXMLConverter);
    config.addComponent('subscript', AnnotationComponent);
    config.addCommand('subscript', AnnotationCommand, { nodeType: 'subscript' });
    config.addTool('subscript', AnnotationTool);
    config.addIcon('subscript', { 'fontawesome': 'fa-subscript' });
    config.addLabel('subscript', {
      en: 'Subscript',
      de: 'Tiefgestellt'
    });
  },
  Subscript: Subscript,
  SubscriptHTMLConverter: SubscriptHTMLConverter
};
