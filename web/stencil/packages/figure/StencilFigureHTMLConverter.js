'use strict';

module.exports = {

  type: 'stencil-figure',
  tagName: 'figure',

  matchElement: function(el) {
    if (el.is('figure')) {
      if (el.find('[data-exec]')) {
        return true
      }
      else return false;
    }
    return false;
  },

  import: function(el, node, converter) {
    var index = el.attr('data-index');
    if (index) {
      node.index = index;
    }

    var caption = el.find('figcaption,caption');
    if(caption){
      node.caption = converter.annotatedText(caption, [node.id, 'caption']);
    }

    var exec = el.find('[data-exec]');
    if(exec){
      node.spec = exec.attr('data-exec');
      node.source = exec.text();
      node.hash = exec.attr('data-hash');
      node.error = exec.attr('data-error');
    }

    var img = el.find('[data-out] img');
    if(img){
      node.image = img.attr('src');
      node.image_style = img.attr('style');
    }

  },

  export: function(node, el, converter) {
    var $$ = converter.$$;

    if(node.index) el.attr('data-index', node.index);

    var caption = $$('figcaption')
      .text(node.caption);
    el.append(caption);

    var exec = $$('pre')
      .attr('data-exec', node.spec)
      .text(node.source);
    if(node.hash) exec.attr('data-hash', node.hash);
    if(node.error) exec.attr('data-error', node.error);
    el.append(exec);

    var img = $$('img')
      .attr('src', node.image);

    var out = $$('div')
      .attr('data-out','true')
      .append(img);
    el.append(out);

  }
};
