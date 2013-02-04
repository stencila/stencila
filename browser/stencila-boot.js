// Stencila boot script loader
var StencilaBoot = (function(){
    "use strict";
    //Define functions for loading different types of resources
    var load = function(tag,rel,type,source,callback){
        var e = document.createElement(tag)
        e.type = type;
        if(rel) e.rel = rel;
        if(tag=='link'){
            e.href = source;
            document.getElementsByTagName("head")[0].appendChild(e);
            if(callback) callback();
        }
        else if(tag=='script'){
            e.src = source;
           if(callback){
                if (e.readyState){  //IE
                    e.onreadystatechange = function(){
                        if (e.readyState == "loaded" ||e.readyState == "complete"){
                            e.onreadystatechange = null;
                            callback();
                        }
                    };
                } else {  //Others
                    e.onload = function(){
                        callback();
                    };
                }
            }
            document.getElementsByTagName("head")[0].appendChild(e);
        }
    }
    var css = function(source,callback) {
        load('link','stylesheet','text/css',source,callback);
    }
    var less = function(source,callback) {
        load('link','stylesheet/less','text/css',source,callback);
    }
    var js = function(source,callback) {
        load('script','','text/javascript',source,callback);
    }

    // Determine if in development or production mode and load files accordingly
    var dev = window.location.hash=="#!dev";
    if(dev){
        if(document.documentElement.className.indexOf('lt-ie9')!=-1){
            js('/components/IE9/index.js');
        }
        js('/components/modernizr/modernizr.js');
        css('/components/normalize-css/normalize.css');
        less('/stencila.less',function(){
            js('/components/less.js/dist/less-1.3.3.js');
        });
        js('/components/jquery/jquery.min.js',function(){
            js('/components/transparency/dist/transparency.min.js');
            js('/components/Base.js-Module/Base.js',function(){
                js('/stencila.js');
            });
        });
    } else {
        css('/stencila.min.css');
        js('/stencila.min.comb.js');
    }
    
    return {
        css: css,
        less: less,
        js: js,
    };
})();
