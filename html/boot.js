/*!
* Stencila bootstrapping Javascript.
*
* A very lightweight collection of functions to embedded in Stencila component HTML
* so they can provide a toolbar with additional functionality when online.
* 
* The Stencila (module)[http://briancray.com/posts/javascript-module-pattern)
* for encapsulating functions.
*/
var _  = Stencila = (function(){

	/************************************************************************************
	 * DOM manipulation
	 * 
	 * These functions provide bare bones alternative to the DOM manipulations functions
	 * in libraries like jQuery and Zepto.
	 ************************************************************************************/
	
	/**
	 * Create DOM elements by parsing an HTML string
	 */
	var make = function(html){
		var div = document.createElement('div');
		div.innerHTML = html;
		return div.childNodes[0];
	}
	
	/**
	 * Create a tag
	 */
	var create = function(tag,attrs,inner){
		attrs = attrs || {};
		var elem = document.createElement(tag);
		for(var key in attrs) elem.setAttribute(key,attrs[key]);
		elem.innerHTML = inner || '';
		return elem;
	}

	/**
	 * Append an element to a node
	 */
	function append(elem,parent){
		(parent || document.body).appendChild(elem);
		return elem;
	}

	/**
	 * Remove and element from a node
	 */
	function remove(elem,parent){
		(parent || document.body).removeChild(elem);
		return elem;
	}

	/**
	 * Add an event listener to an element
	 */
	function on(elem,event,func){
		elem.addEventListener(event,func);
		return elem;
	}

	/**
	 * CSS selector query
	 */
	var select = function(query,node){
		return (node || document).querySelector(query);
	}

	/************************************************************************************
	 * Resource loading
	 *
	 * A simple JS/CSS/whatevever loading function
	 ***********************************************************************************/

    /**
     * Load a resource
     */
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
    /**
     * Load CSS
     */
    var css = function(source,callback) {
        load('link','stylesheet','text/css',source,callback);
    }
    /**
     * Load Javascript
     */
    var js = function(source,callback) {
        load('script','','text/javascript',source,callback);
    }


	/************************************************************************************
	 * Asynchronous communications with Stencila
	 *
	 * A hidden iframe is used to POST to hub.stenci.la (without the parent frame being refreshed).
	 * The server responds with a very small <script> with a postMessage(...) in it.
	 * The message is received by the main document and actions taken.
	 * This approach is ["Cross document messaging"](http://en.wikipedia.org/wiki/Same-origin_policy#Cross-document_messaging)
	 * which is one method for relaxing a browser's Same Origin Policy.
	 *
	 * The other three alternatives to relaxing the Same Origin Policy are not appropriate
	 * in this context. In partcular Cross-Origin Resource Sharing (CORS) will not work
	 * if a component is on a local file. In those cases the `Origin` header is null which can not be authorized.
	 ***********************************************************************************/

	/**
	 * The hidden iframe used for connections
	 */
	var hub;

	/**
	 * Establish a connection to Stencila by creating a hidden iframe.
	 *
	 * The iframe's name must be the same as specified in the form's target.
	 * The iframe is hidden using the HTML5 hidden atribute as well as other attributes for legacy 
	 * browsers as suggested by http://blog.paciellogroup.com/2010/04/making-sure-hidden-frames-are-hidden/
	 */
	var connect = function(){
		hub = append(create('iframe',{
			name: "hub",
			src: "http://hub.stenci.la/connect",
			hidden: "true",
			width: "0",
			height: "0",
			tabindex: "-1",
			style: "display:none",
		}));

		window.addEventListener('message',receive,false);
		setInterval(send,10000);
	}

	/**
	 * Send a request to Stencila via form POSTing in iframe
	 */
	function send(page,params){
		page = page || "connect";
		params = params || {};
		var form = create("form",{
			method: "post",
			enctype:"multipart/form-data", // To allow for file uploads
			action: "http://hub.stenci.la/"+page,
			target: "hub",
		});
		for(var key in params) {
			form.appendChild(create("input",{
				type: "hidden",
				name: key,
				value: params[key],
			}));
		}
		append(form);
		form.submit();
		remove(form);
	}

	/**
	 * Receive a response from Stencila via a postMessage() call in iframe
	 */
	function receive(event){
		console.log(event);
		// Clear the hub iframe's content so that it no longer contains
		// a postMessage call. Otherwise when hitting the back button after a
		// opening a new window, the message gets received again.
		hub.innerHTML = "";
		if(event.origin==='http://hub.stenci.la'){
			var parts = event.data.split(' ');
			if(parts[0]=='ok'){
				connection(true);
			} else if(parts[0]=='open'){
				open(event.origin+'/'+parts[1]);
			}
		}
	}

	/**
	 * Set the connection status
	 */
	function connection(on){
		if(on) select("#connection").className = "on";
		else select("#connection").className = "off";
	}

	/**
	 * Open a URL as instructed by a postMessage
	 */
	function open(url){
		// Using window.open can cause a new window to open and get blocked by the browser as
		// a pop-up.
		// See http://stackoverflow.com/questions/4907843/open-url-in-new-tab-using-javascript
		//window.open(url,'_blank');
		// Setting window location is an alternative but can cause issues with the back buton
		window.location.href = url;
	}

	/**
	 * Set up the Stencila toolbar for the particular type of component
	 */
	function toolbar(type){
		var tools = select('.tools',append(make(
			'<div id="toolbar"><div class="slider">'+
				'<div class="handle"></div>'+
				'<div class="tools">'+
					'<p><a class="load" href="#load">Load</a></p>'+
					'<p><a class="point" target="_blank" href="http://hub.stenci.la/point?'+
						'&type=' + type +
						'&id=' + select("meta[name=id]").getAttribute('content') +
						'&url=' + encodeURIComponent(window.location.href)+'">Point</a></p>' +
				'</div>'+
				'<div id="connection" class="off">Unable to connect to <a href="http://hub.stenci.la">Stencila</a></div>'+
			'</div></div>'
		)));
		on(select('a.load',tools),'click',function(event){
			event.preventDefault();
			send('load',{
				'html': '<html>'+document.documentElement.innerHTML+'</html>'
			});
		});
	}

    // Functions exported by the Stencila module
    return {
        load:load,css:css,less:less,js:js,
        connect:connect,send:send,receive:receive,
        toolbar:toolbar
    };
    
})();
