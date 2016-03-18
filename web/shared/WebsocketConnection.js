/**
 * Websocket connection to a component running in a session
 * on localhost or some other server
 *
 * Uses the WAMP (http://wamp.ws) messaging protocol
 */
function WebsocketConnection(url){
    // Callbacks registered for remote procedure calls (see call() method)
    this.callbacks = {};
    // Identifier for messages; incremented in call method
    this.id = 0;
    // Actual websocket
    this.socket = null;

    this.connect(url);
}

/**
 * Connect to remote
 * 
 * @param  {String} url URL to connect to
 */
WebsocketConnection.prototype.connect = function(url){
    var self = this;
    // Create a new websocket
    self.socket = new WebSocket(url);
    // Bind some socket events
    //   when connection is opened...
    self.socket.onopen = function(event){
        self.ok = true;
    };
    //   when there are any connection errors...
    self.socket.onclose = function(event){
        console.warn("WebsocketConnection closed");
        self.ok = false;
    };
    //   when a message is recieved...
    self.socket.onmessage = function(event){
        self.receive(event.data);
    };
    // Automatically disconnect when page is unloaded
    window.addEventListener("beforeunload", function(event){
        self.disconnect();
    });
}

/**
 * Wait until the socket is ready before doing
 * something on it (usually a send)
 */
WebsocketConnection.prototype.wait = function(callback){
    setTimeout(
        function () {
            if (this.socket.readyState === 1) {
                if(callback){
                    callback();
                }
                return
            } else {
                this.wait(callback);
            }
        }.bind(this),
        5
    );
}

/**
 * Disconnect from remote
 */
WebsocketConnection.prototype.disconnect = function(){
    this.socket.close();
}

/**
 * Send data to remote
 */
WebsocketConnection.prototype.send = function(data){
    this.wait(function(){
        this.socket.send(data);
    }.bind(this));
}

/**
 * Receive a message from remote
 * 
 * @param  {String} data
 */
WebsocketConnection.prototype.receive = function(data){
    // Parse JSON
    var message = [8];
    try {
        message = JSON.parse(data);
    }
    catch(error) {
        throw 'WebsocketConnection.receive. Error parsing WAMP message data.\n  data:'+data+'\n  error:'+error;
    }
    // Act on WAMP code
    var code = message[0];
    if(code==50){
        // [RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list]
        var id = message[1];
        console.info('WebsocketConnection.receive. id: '+this.id);
        var callback = this.callbacks[id];
        if(callback){
            var results = message[3];
            // WAMP allows for muliple returns
            // Only passing on a single result, the first
            var result = results[0];
            callback(result);
        }
    }
    else if(code==8){
        throw message;
    }
    else {
        throw "WebsocketConnection.receive. WAMP message type unknown/unhandled:"+code;
    }
}

/**
 * Make a remote procedure call
 * See https://github.com/tavendo/WAMP/blob/master/spec/basic.md#call-1
 * 
 * @param  {String}   method   Name of method to call
 * @param  {Array}    args     Array of arguments
 * @param  {Function} callback Function to call when method returns (potentially with a result)
 */
WebsocketConnection.prototype.call = function(method,args,callback){
    if(arguments.length==1){
        args = [];
        callback = undefined;
    }
    else if(arguments.length==2){
        args = [];
        callback = arguments[1];
    }
    // Increment id
    // According to https://github.com/tavendo/WAMP/blob/master/spec/basic.md#ids
    // "IDs in the session scope SHOULD be incremented by 1 beginning with 1"
    this.id++;
    // Generate a WAMP call array
    var wamp = [
        48,         // CALL
        this.id,    // Request|id
        {},         // Options|dict
        method,     // Procedure|uri
        args        // Arguments|list
    ];
    // Register callback
    if(callback){
        this.callbacks[this.id] = callback;
    }
    // Send WAMP
    console.info('WebsocketConnection.call. id: '+this.id+' method: '+method);
    this.send(JSON.stringify(wamp));
}

module.exports = WebsocketConnection;
