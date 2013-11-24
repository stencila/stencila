//Default Javascript for Stencila stencils

(function() {
    var e = document.createElement('script');
    e.type = 'text/javascript';
    e.async = true;
    e.src = 'http://cdn.mathjax.org/mathjax/latest/MathJax.js?config=AM_HTMLorMML';
    document.getElementsByTagName("head")[0].appendChild(e);
})();
