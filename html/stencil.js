// Load updated CSS and JS
_.css('http://get.stenci.la/html/stencil.min.css');
_.js('http://get.stenci.la/html/stencil.min.js');

// Load MathJAX. This should really only be done if there is math in the stencil
_.js('http://cdn.mathjax.org/mathjax/latest/MathJax.js?config=AM_HTMLorMML');

// Setup toolbar
_.connect();
_.toolbar("stencil");
