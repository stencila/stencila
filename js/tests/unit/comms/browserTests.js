'use strict';

var commonjsGlobal = typeof window !== 'undefined' ? window : typeof global !== 'undefined' ? global : typeof self !== 'undefined' ? self : {};

function createCommonjsModule(fn, module) {
	return module = { exports: {} }, fn(module, module.exports), module.exports;
}

var browserPonyfill = createCommonjsModule(function (module) {
var __root__ = (function (root) {
function F() { this.fetch = false; }
F.prototype = root;
return new F();
})(typeof self !== 'undefined' ? self : commonjsGlobal);
(function(self) {

(function(self) {

  if (self.fetch) {
    return
  }

  var support = {
    searchParams: 'URLSearchParams' in self,
    iterable: 'Symbol' in self && 'iterator' in Symbol,
    blob: 'FileReader' in self && 'Blob' in self && (function() {
      try {
        new Blob();
        return true
      } catch(e) {
        return false
      }
    })(),
    formData: 'FormData' in self,
    arrayBuffer: 'ArrayBuffer' in self
  };

  if (support.arrayBuffer) {
    var viewClasses = [
      '[object Int8Array]',
      '[object Uint8Array]',
      '[object Uint8ClampedArray]',
      '[object Int16Array]',
      '[object Uint16Array]',
      '[object Int32Array]',
      '[object Uint32Array]',
      '[object Float32Array]',
      '[object Float64Array]'
    ];

    var isDataView = function(obj) {
      return obj && DataView.prototype.isPrototypeOf(obj)
    };

    var isArrayBufferView = ArrayBuffer.isView || function(obj) {
      return obj && viewClasses.indexOf(Object.prototype.toString.call(obj)) > -1
    };
  }

  function normalizeName(name) {
    if (typeof name !== 'string') {
      name = String(name);
    }
    if (/[^a-z0-9\-#$%&'*+.\^_`|~]/i.test(name)) {
      throw new TypeError('Invalid character in header field name')
    }
    return name.toLowerCase()
  }

  function normalizeValue(value) {
    if (typeof value !== 'string') {
      value = String(value);
    }
    return value
  }

  // Build a destructive iterator for the value list
  function iteratorFor(items) {
    var iterator = {
      next: function() {
        var value = items.shift();
        return {done: value === undefined, value: value}
      }
    };

    if (support.iterable) {
      iterator[Symbol.iterator] = function() {
        return iterator
      };
    }

    return iterator
  }

  function Headers(headers) {
    this.map = {};

    if (headers instanceof Headers) {
      headers.forEach(function(value, name) {
        this.append(name, value);
      }, this);
    } else if (Array.isArray(headers)) {
      headers.forEach(function(header) {
        this.append(header[0], header[1]);
      }, this);
    } else if (headers) {
      Object.getOwnPropertyNames(headers).forEach(function(name) {
        this.append(name, headers[name]);
      }, this);
    }
  }

  Headers.prototype.append = function(name, value) {
    name = normalizeName(name);
    value = normalizeValue(value);
    var oldValue = this.map[name];
    this.map[name] = oldValue ? oldValue+','+value : value;
  };

  Headers.prototype['delete'] = function(name) {
    delete this.map[normalizeName(name)];
  };

  Headers.prototype.get = function(name) {
    name = normalizeName(name);
    return this.has(name) ? this.map[name] : null
  };

  Headers.prototype.has = function(name) {
    return this.map.hasOwnProperty(normalizeName(name))
  };

  Headers.prototype.set = function(name, value) {
    this.map[normalizeName(name)] = normalizeValue(value);
  };

  Headers.prototype.forEach = function(callback, thisArg) {
    for (var name in this.map) {
      if (this.map.hasOwnProperty(name)) {
        callback.call(thisArg, this.map[name], name, this);
      }
    }
  };

  Headers.prototype.keys = function() {
    var items = [];
    this.forEach(function(value, name) { items.push(name); });
    return iteratorFor(items)
  };

  Headers.prototype.values = function() {
    var items = [];
    this.forEach(function(value) { items.push(value); });
    return iteratorFor(items)
  };

  Headers.prototype.entries = function() {
    var items = [];
    this.forEach(function(value, name) { items.push([name, value]); });
    return iteratorFor(items)
  };

  if (support.iterable) {
    Headers.prototype[Symbol.iterator] = Headers.prototype.entries;
  }

  function consumed(body) {
    if (body.bodyUsed) {
      return Promise.reject(new TypeError('Already read'))
    }
    body.bodyUsed = true;
  }

  function fileReaderReady(reader) {
    return new Promise(function(resolve, reject) {
      reader.onload = function() {
        resolve(reader.result);
      };
      reader.onerror = function() {
        reject(reader.error);
      };
    })
  }

  function readBlobAsArrayBuffer(blob) {
    var reader = new FileReader();
    var promise = fileReaderReady(reader);
    reader.readAsArrayBuffer(blob);
    return promise
  }

  function readBlobAsText(blob) {
    var reader = new FileReader();
    var promise = fileReaderReady(reader);
    reader.readAsText(blob);
    return promise
  }

  function readArrayBufferAsText(buf) {
    var view = new Uint8Array(buf);
    var chars = new Array(view.length);

    for (var i = 0; i < view.length; i++) {
      chars[i] = String.fromCharCode(view[i]);
    }
    return chars.join('')
  }

  function bufferClone(buf) {
    if (buf.slice) {
      return buf.slice(0)
    } else {
      var view = new Uint8Array(buf.byteLength);
      view.set(new Uint8Array(buf));
      return view.buffer
    }
  }

  function Body() {
    this.bodyUsed = false;

    this._initBody = function(body) {
      this._bodyInit = body;
      if (!body) {
        this._bodyText = '';
      } else if (typeof body === 'string') {
        this._bodyText = body;
      } else if (support.blob && Blob.prototype.isPrototypeOf(body)) {
        this._bodyBlob = body;
      } else if (support.formData && FormData.prototype.isPrototypeOf(body)) {
        this._bodyFormData = body;
      } else if (support.searchParams && URLSearchParams.prototype.isPrototypeOf(body)) {
        this._bodyText = body.toString();
      } else if (support.arrayBuffer && support.blob && isDataView(body)) {
        this._bodyArrayBuffer = bufferClone(body.buffer);
        // IE 10-11 can't handle a DataView body.
        this._bodyInit = new Blob([this._bodyArrayBuffer]);
      } else if (support.arrayBuffer && (ArrayBuffer.prototype.isPrototypeOf(body) || isArrayBufferView(body))) {
        this._bodyArrayBuffer = bufferClone(body);
      } else {
        throw new Error('unsupported BodyInit type')
      }

      if (!this.headers.get('content-type')) {
        if (typeof body === 'string') {
          this.headers.set('content-type', 'text/plain;charset=UTF-8');
        } else if (this._bodyBlob && this._bodyBlob.type) {
          this.headers.set('content-type', this._bodyBlob.type);
        } else if (support.searchParams && URLSearchParams.prototype.isPrototypeOf(body)) {
          this.headers.set('content-type', 'application/x-www-form-urlencoded;charset=UTF-8');
        }
      }
    };

    if (support.blob) {
      this.blob = function() {
        var rejected = consumed(this);
        if (rejected) {
          return rejected
        }

        if (this._bodyBlob) {
          return Promise.resolve(this._bodyBlob)
        } else if (this._bodyArrayBuffer) {
          return Promise.resolve(new Blob([this._bodyArrayBuffer]))
        } else if (this._bodyFormData) {
          throw new Error('could not read FormData body as blob')
        } else {
          return Promise.resolve(new Blob([this._bodyText]))
        }
      };

      this.arrayBuffer = function() {
        if (this._bodyArrayBuffer) {
          return consumed(this) || Promise.resolve(this._bodyArrayBuffer)
        } else {
          return this.blob().then(readBlobAsArrayBuffer)
        }
      };
    }

    this.text = function() {
      var rejected = consumed(this);
      if (rejected) {
        return rejected
      }

      if (this._bodyBlob) {
        return readBlobAsText(this._bodyBlob)
      } else if (this._bodyArrayBuffer) {
        return Promise.resolve(readArrayBufferAsText(this._bodyArrayBuffer))
      } else if (this._bodyFormData) {
        throw new Error('could not read FormData body as text')
      } else {
        return Promise.resolve(this._bodyText)
      }
    };

    if (support.formData) {
      this.formData = function() {
        return this.text().then(decode)
      };
    }

    this.json = function() {
      return this.text().then(JSON.parse)
    };

    return this
  }

  // HTTP methods whose capitalization should be normalized
  var methods = ['DELETE', 'GET', 'HEAD', 'OPTIONS', 'POST', 'PUT'];

  function normalizeMethod(method) {
    var upcased = method.toUpperCase();
    return (methods.indexOf(upcased) > -1) ? upcased : method
  }

  function Request(input, options) {
    options = options || {};
    var body = options.body;

    if (input instanceof Request) {
      if (input.bodyUsed) {
        throw new TypeError('Already read')
      }
      this.url = input.url;
      this.credentials = input.credentials;
      if (!options.headers) {
        this.headers = new Headers(input.headers);
      }
      this.method = input.method;
      this.mode = input.mode;
      if (!body && input._bodyInit != null) {
        body = input._bodyInit;
        input.bodyUsed = true;
      }
    } else {
      this.url = String(input);
    }

    this.credentials = options.credentials || this.credentials || 'omit';
    if (options.headers || !this.headers) {
      this.headers = new Headers(options.headers);
    }
    this.method = normalizeMethod(options.method || this.method || 'GET');
    this.mode = options.mode || this.mode || null;
    this.referrer = null;

    if ((this.method === 'GET' || this.method === 'HEAD') && body) {
      throw new TypeError('Body not allowed for GET or HEAD requests')
    }
    this._initBody(body);
  }

  Request.prototype.clone = function() {
    return new Request(this, { body: this._bodyInit })
  };

  function decode(body) {
    var form = new FormData();
    body.trim().split('&').forEach(function(bytes) {
      if (bytes) {
        var split = bytes.split('=');
        var name = split.shift().replace(/\+/g, ' ');
        var value = split.join('=').replace(/\+/g, ' ');
        form.append(decodeURIComponent(name), decodeURIComponent(value));
      }
    });
    return form
  }

  function parseHeaders(rawHeaders) {
    var headers = new Headers();
    // Replace instances of \r\n and \n followed by at least one space or horizontal tab with a space
    // https://tools.ietf.org/html/rfc7230#section-3.2
    var preProcessedHeaders = rawHeaders.replace(/\r?\n[\t ]+/g, ' ');
    preProcessedHeaders.split(/\r?\n/).forEach(function(line) {
      var parts = line.split(':');
      var key = parts.shift().trim();
      if (key) {
        var value = parts.join(':').trim();
        headers.append(key, value);
      }
    });
    return headers
  }

  Body.call(Request.prototype);

  function Response(bodyInit, options) {
    if (!options) {
      options = {};
    }

    this.type = 'default';
    this.status = options.status === undefined ? 200 : options.status;
    this.ok = this.status >= 200 && this.status < 300;
    this.statusText = 'statusText' in options ? options.statusText : 'OK';
    this.headers = new Headers(options.headers);
    this.url = options.url || '';
    this._initBody(bodyInit);
  }

  Body.call(Response.prototype);

  Response.prototype.clone = function() {
    return new Response(this._bodyInit, {
      status: this.status,
      statusText: this.statusText,
      headers: new Headers(this.headers),
      url: this.url
    })
  };

  Response.error = function() {
    var response = new Response(null, {status: 0, statusText: ''});
    response.type = 'error';
    return response
  };

  var redirectStatuses = [301, 302, 303, 307, 308];

  Response.redirect = function(url, status) {
    if (redirectStatuses.indexOf(status) === -1) {
      throw new RangeError('Invalid status code')
    }

    return new Response(null, {status: status, headers: {location: url}})
  };

  self.Headers = Headers;
  self.Request = Request;
  self.Response = Response;

  self.fetch = function(input, init) {
    return new Promise(function(resolve, reject) {
      var request = new Request(input, init);
      var xhr = new XMLHttpRequest();

      xhr.onload = function() {
        var options = {
          status: xhr.status,
          statusText: xhr.statusText,
          headers: parseHeaders(xhr.getAllResponseHeaders() || '')
        };
        options.url = 'responseURL' in xhr ? xhr.responseURL : options.headers.get('X-Request-URL');
        var body = 'response' in xhr ? xhr.response : xhr.responseText;
        resolve(new Response(body, options));
      };

      xhr.onerror = function() {
        reject(new TypeError('Network request failed'));
      };

      xhr.ontimeout = function() {
        reject(new TypeError('Network request failed'));
      };

      xhr.open(request.method, request.url, true);

      if (request.credentials === 'include') {
        xhr.withCredentials = true;
      } else if (request.credentials === 'omit') {
        xhr.withCredentials = false;
      }

      if ('responseType' in xhr && support.blob) {
        xhr.responseType = 'blob';
      }

      request.headers.forEach(function(value, name) {
        xhr.setRequestHeader(name, value);
      });

      xhr.send(typeof request._bodyInit === 'undefined' ? null : request._bodyInit);
    })
  };
  self.fetch.polyfill = true;
})(typeof self !== 'undefined' ? self : this);
}).call(__root__, void(0));
var fetch = __root__.fetch;
var Response = fetch.Response = __root__.Response;
var Request = fetch.Request = __root__.Request;
var Headers = fetch.Headers = __root__.Headers;
if (module.exports) {
module.exports = fetch;
// Needed for TypeScript consumers without esModuleInterop.
module.exports.default = fetch;
}
});

/**
 * A JSON-RPC 2.0 request
 *
 * @see {@link https://www.jsonrpc.org/specification#request_object}
 */
class JsonRpcRequest {
    constructor(method, params) {
        /**
         * A string specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
         */
        this.jsonrpc = '2.0';
        JsonRpcRequest.counter += 1;
        this.id = JsonRpcRequest.counter;
        this.method = method;
        this.params = params;
    }
}
/**
 * A counter for generating unique, sequential request ids.
 *
 * Request ids don't need to be sequential but this helps with debugging.
 * Request ids don't need to be unique across clients.
 */
JsonRpcRequest.counter = 0;

/*! *****************************************************************************
Copyright (c) Microsoft Corporation. All rights reserved.
Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at http://www.apache.org/licenses/LICENSE-2.0

THIS CODE IS PROVIDED ON AN *AS IS* BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, EITHER EXPRESS OR IMPLIED, INCLUDING WITHOUT LIMITATION ANY IMPLIED
WARRANTIES OR CONDITIONS OF TITLE, FITNESS FOR A PARTICULAR PURPOSE,
MERCHANTABLITY OR NON-INFRINGEMENT.

See the Apache Version 2.0 License for specific language governing permissions
and limitations under the License.
***************************************************************************** */

function __decorate(decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
}

function __metadata(metadataKey, metadataValue) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(metadataKey, metadataValue);
}

/*! *****************************************************************************
Copyright (C) Microsoft. All rights reserved.
Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at http://www.apache.org/licenses/LICENSE-2.0

THIS CODE IS PROVIDED ON AN *AS IS* BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, EITHER EXPRESS OR IMPLIED, INCLUDING WITHOUT LIMITATION ANY IMPLIED
WARRANTIES OR CONDITIONS OF TITLE, FITNESS FOR A PARTICULAR PURPOSE,
MERCHANTABLITY OR NON-INFRINGEMENT.

See the Apache Version 2.0 License for specific language governing permissions
and limitations under the License.
***************************************************************************** */
var Reflect$1;
(function (Reflect) {
    // Metadata Proposal
    // https://rbuckton.github.io/reflect-metadata/
    (function (factory) {
        var root = typeof commonjsGlobal === "object" ? commonjsGlobal :
            typeof self === "object" ? self :
                typeof this === "object" ? this :
                    Function("return this;")();
        var exporter = makeExporter(Reflect);
        if (typeof root.Reflect === "undefined") {
            root.Reflect = Reflect;
        }
        else {
            exporter = makeExporter(root.Reflect, exporter);
        }
        factory(exporter);
        function makeExporter(target, previous) {
            return function (key, value) {
                if (typeof target[key] !== "function") {
                    Object.defineProperty(target, key, { configurable: true, writable: true, value: value });
                }
                if (previous)
                    previous(key, value);
            };
        }
    })(function (exporter) {
        var hasOwn = Object.prototype.hasOwnProperty;
        // feature test for Symbol support
        var supportsSymbol = typeof Symbol === "function";
        var toPrimitiveSymbol = supportsSymbol && typeof Symbol.toPrimitive !== "undefined" ? Symbol.toPrimitive : "@@toPrimitive";
        var iteratorSymbol = supportsSymbol && typeof Symbol.iterator !== "undefined" ? Symbol.iterator : "@@iterator";
        var supportsCreate = typeof Object.create === "function"; // feature test for Object.create support
        var supportsProto = { __proto__: [] } instanceof Array; // feature test for __proto__ support
        var downLevel = !supportsCreate && !supportsProto;
        var HashMap = {
            // create an object in dictionary mode (a.k.a. "slow" mode in v8)
            create: supportsCreate
                ? function () { return MakeDictionary(Object.create(null)); }
                : supportsProto
                    ? function () { return MakeDictionary({ __proto__: null }); }
                    : function () { return MakeDictionary({}); },
            has: downLevel
                ? function (map, key) { return hasOwn.call(map, key); }
                : function (map, key) { return key in map; },
            get: downLevel
                ? function (map, key) { return hasOwn.call(map, key) ? map[key] : undefined; }
                : function (map, key) { return map[key]; },
        };
        // Load global or shim versions of Map, Set, and WeakMap
        var functionPrototype = Object.getPrototypeOf(Function);
        var usePolyfill = typeof process === "object" && process.env && process.env["REFLECT_METADATA_USE_MAP_POLYFILL"] === "true";
        var _Map = !usePolyfill && typeof Map === "function" && typeof Map.prototype.entries === "function" ? Map : CreateMapPolyfill();
        var _Set = !usePolyfill && typeof Set === "function" && typeof Set.prototype.entries === "function" ? Set : CreateSetPolyfill();
        var _WeakMap = !usePolyfill && typeof WeakMap === "function" ? WeakMap : CreateWeakMapPolyfill();
        // [[Metadata]] internal slot
        // https://rbuckton.github.io/reflect-metadata/#ordinary-object-internal-methods-and-internal-slots
        var Metadata = new _WeakMap();
        /**
         * Applies a set of decorators to a property of a target object.
         * @param decorators An array of decorators.
         * @param target The target object.
         * @param propertyKey (Optional) The property key to decorate.
         * @param attributes (Optional) The property descriptor for the target key.
         * @remarks Decorators are applied in reverse order.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     Example = Reflect.decorate(decoratorsArray, Example);
         *
         *     // property (on constructor)
         *     Reflect.decorate(decoratorsArray, Example, "staticProperty");
         *
         *     // property (on prototype)
         *     Reflect.decorate(decoratorsArray, Example.prototype, "property");
         *
         *     // method (on constructor)
         *     Object.defineProperty(Example, "staticMethod",
         *         Reflect.decorate(decoratorsArray, Example, "staticMethod",
         *             Object.getOwnPropertyDescriptor(Example, "staticMethod")));
         *
         *     // method (on prototype)
         *     Object.defineProperty(Example.prototype, "method",
         *         Reflect.decorate(decoratorsArray, Example.prototype, "method",
         *             Object.getOwnPropertyDescriptor(Example.prototype, "method")));
         *
         */
        function decorate(decorators, target, propertyKey, attributes) {
            if (!IsUndefined(propertyKey)) {
                if (!IsArray(decorators))
                    throw new TypeError();
                if (!IsObject(target))
                    throw new TypeError();
                if (!IsObject(attributes) && !IsUndefined(attributes) && !IsNull(attributes))
                    throw new TypeError();
                if (IsNull(attributes))
                    attributes = undefined;
                propertyKey = ToPropertyKey(propertyKey);
                return DecorateProperty(decorators, target, propertyKey, attributes);
            }
            else {
                if (!IsArray(decorators))
                    throw new TypeError();
                if (!IsConstructor(target))
                    throw new TypeError();
                return DecorateConstructor(decorators, target);
            }
        }
        exporter("decorate", decorate);
        // 4.1.2 Reflect.metadata(metadataKey, metadataValue)
        // https://rbuckton.github.io/reflect-metadata/#reflect.metadata
        /**
         * A default metadata decorator factory that can be used on a class, class member, or parameter.
         * @param metadataKey The key for the metadata entry.
         * @param metadataValue The value for the metadata entry.
         * @returns A decorator function.
         * @remarks
         * If `metadataKey` is already defined for the target and target key, the
         * metadataValue for that key will be overwritten.
         * @example
         *
         *     // constructor
         *     @Reflect.metadata(key, value)
         *     class Example {
         *     }
         *
         *     // property (on constructor, TypeScript only)
         *     class Example {
         *         @Reflect.metadata(key, value)
         *         static staticProperty;
         *     }
         *
         *     // property (on prototype, TypeScript only)
         *     class Example {
         *         @Reflect.metadata(key, value)
         *         property;
         *     }
         *
         *     // method (on constructor)
         *     class Example {
         *         @Reflect.metadata(key, value)
         *         static staticMethod() { }
         *     }
         *
         *     // method (on prototype)
         *     class Example {
         *         @Reflect.metadata(key, value)
         *         method() { }
         *     }
         *
         */
        function metadata(metadataKey, metadataValue) {
            function decorator(target, propertyKey) {
                if (!IsObject(target))
                    throw new TypeError();
                if (!IsUndefined(propertyKey) && !IsPropertyKey(propertyKey))
                    throw new TypeError();
                OrdinaryDefineOwnMetadata(metadataKey, metadataValue, target, propertyKey);
            }
            return decorator;
        }
        exporter("metadata", metadata);
        /**
         * Define a unique metadata entry on the target.
         * @param metadataKey A key used to store and retrieve metadata.
         * @param metadataValue A value that contains attached metadata.
         * @param target The target object on which to define metadata.
         * @param propertyKey (Optional) The property key for the target.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     Reflect.defineMetadata("custom:annotation", options, Example);
         *
         *     // property (on constructor)
         *     Reflect.defineMetadata("custom:annotation", options, Example, "staticProperty");
         *
         *     // property (on prototype)
         *     Reflect.defineMetadata("custom:annotation", options, Example.prototype, "property");
         *
         *     // method (on constructor)
         *     Reflect.defineMetadata("custom:annotation", options, Example, "staticMethod");
         *
         *     // method (on prototype)
         *     Reflect.defineMetadata("custom:annotation", options, Example.prototype, "method");
         *
         *     // decorator factory as metadata-producing annotation.
         *     function MyAnnotation(options): Decorator {
         *         return (target, key?) => Reflect.defineMetadata("custom:annotation", options, target, key);
         *     }
         *
         */
        function defineMetadata(metadataKey, metadataValue, target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            return OrdinaryDefineOwnMetadata(metadataKey, metadataValue, target, propertyKey);
        }
        exporter("defineMetadata", defineMetadata);
        /**
         * Gets a value indicating whether the target object or its prototype chain has the provided metadata key defined.
         * @param metadataKey A key used to store and retrieve metadata.
         * @param target The target object on which the metadata is defined.
         * @param propertyKey (Optional) The property key for the target.
         * @returns `true` if the metadata key was defined on the target object or its prototype chain; otherwise, `false`.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     result = Reflect.hasMetadata("custom:annotation", Example);
         *
         *     // property (on constructor)
         *     result = Reflect.hasMetadata("custom:annotation", Example, "staticProperty");
         *
         *     // property (on prototype)
         *     result = Reflect.hasMetadata("custom:annotation", Example.prototype, "property");
         *
         *     // method (on constructor)
         *     result = Reflect.hasMetadata("custom:annotation", Example, "staticMethod");
         *
         *     // method (on prototype)
         *     result = Reflect.hasMetadata("custom:annotation", Example.prototype, "method");
         *
         */
        function hasMetadata(metadataKey, target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            return OrdinaryHasMetadata(metadataKey, target, propertyKey);
        }
        exporter("hasMetadata", hasMetadata);
        /**
         * Gets a value indicating whether the target object has the provided metadata key defined.
         * @param metadataKey A key used to store and retrieve metadata.
         * @param target The target object on which the metadata is defined.
         * @param propertyKey (Optional) The property key for the target.
         * @returns `true` if the metadata key was defined on the target object; otherwise, `false`.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     result = Reflect.hasOwnMetadata("custom:annotation", Example);
         *
         *     // property (on constructor)
         *     result = Reflect.hasOwnMetadata("custom:annotation", Example, "staticProperty");
         *
         *     // property (on prototype)
         *     result = Reflect.hasOwnMetadata("custom:annotation", Example.prototype, "property");
         *
         *     // method (on constructor)
         *     result = Reflect.hasOwnMetadata("custom:annotation", Example, "staticMethod");
         *
         *     // method (on prototype)
         *     result = Reflect.hasOwnMetadata("custom:annotation", Example.prototype, "method");
         *
         */
        function hasOwnMetadata(metadataKey, target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            return OrdinaryHasOwnMetadata(metadataKey, target, propertyKey);
        }
        exporter("hasOwnMetadata", hasOwnMetadata);
        /**
         * Gets the metadata value for the provided metadata key on the target object or its prototype chain.
         * @param metadataKey A key used to store and retrieve metadata.
         * @param target The target object on which the metadata is defined.
         * @param propertyKey (Optional) The property key for the target.
         * @returns The metadata value for the metadata key if found; otherwise, `undefined`.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     result = Reflect.getMetadata("custom:annotation", Example);
         *
         *     // property (on constructor)
         *     result = Reflect.getMetadata("custom:annotation", Example, "staticProperty");
         *
         *     // property (on prototype)
         *     result = Reflect.getMetadata("custom:annotation", Example.prototype, "property");
         *
         *     // method (on constructor)
         *     result = Reflect.getMetadata("custom:annotation", Example, "staticMethod");
         *
         *     // method (on prototype)
         *     result = Reflect.getMetadata("custom:annotation", Example.prototype, "method");
         *
         */
        function getMetadata(metadataKey, target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            return OrdinaryGetMetadata(metadataKey, target, propertyKey);
        }
        exporter("getMetadata", getMetadata);
        /**
         * Gets the metadata value for the provided metadata key on the target object.
         * @param metadataKey A key used to store and retrieve metadata.
         * @param target The target object on which the metadata is defined.
         * @param propertyKey (Optional) The property key for the target.
         * @returns The metadata value for the metadata key if found; otherwise, `undefined`.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     result = Reflect.getOwnMetadata("custom:annotation", Example);
         *
         *     // property (on constructor)
         *     result = Reflect.getOwnMetadata("custom:annotation", Example, "staticProperty");
         *
         *     // property (on prototype)
         *     result = Reflect.getOwnMetadata("custom:annotation", Example.prototype, "property");
         *
         *     // method (on constructor)
         *     result = Reflect.getOwnMetadata("custom:annotation", Example, "staticMethod");
         *
         *     // method (on prototype)
         *     result = Reflect.getOwnMetadata("custom:annotation", Example.prototype, "method");
         *
         */
        function getOwnMetadata(metadataKey, target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            return OrdinaryGetOwnMetadata(metadataKey, target, propertyKey);
        }
        exporter("getOwnMetadata", getOwnMetadata);
        /**
         * Gets the metadata keys defined on the target object or its prototype chain.
         * @param target The target object on which the metadata is defined.
         * @param propertyKey (Optional) The property key for the target.
         * @returns An array of unique metadata keys.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     result = Reflect.getMetadataKeys(Example);
         *
         *     // property (on constructor)
         *     result = Reflect.getMetadataKeys(Example, "staticProperty");
         *
         *     // property (on prototype)
         *     result = Reflect.getMetadataKeys(Example.prototype, "property");
         *
         *     // method (on constructor)
         *     result = Reflect.getMetadataKeys(Example, "staticMethod");
         *
         *     // method (on prototype)
         *     result = Reflect.getMetadataKeys(Example.prototype, "method");
         *
         */
        function getMetadataKeys(target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            return OrdinaryMetadataKeys(target, propertyKey);
        }
        exporter("getMetadataKeys", getMetadataKeys);
        /**
         * Gets the unique metadata keys defined on the target object.
         * @param target The target object on which the metadata is defined.
         * @param propertyKey (Optional) The property key for the target.
         * @returns An array of unique metadata keys.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     result = Reflect.getOwnMetadataKeys(Example);
         *
         *     // property (on constructor)
         *     result = Reflect.getOwnMetadataKeys(Example, "staticProperty");
         *
         *     // property (on prototype)
         *     result = Reflect.getOwnMetadataKeys(Example.prototype, "property");
         *
         *     // method (on constructor)
         *     result = Reflect.getOwnMetadataKeys(Example, "staticMethod");
         *
         *     // method (on prototype)
         *     result = Reflect.getOwnMetadataKeys(Example.prototype, "method");
         *
         */
        function getOwnMetadataKeys(target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            return OrdinaryOwnMetadataKeys(target, propertyKey);
        }
        exporter("getOwnMetadataKeys", getOwnMetadataKeys);
        /**
         * Deletes the metadata entry from the target object with the provided key.
         * @param metadataKey A key used to store and retrieve metadata.
         * @param target The target object on which the metadata is defined.
         * @param propertyKey (Optional) The property key for the target.
         * @returns `true` if the metadata entry was found and deleted; otherwise, false.
         * @example
         *
         *     class Example {
         *         // property declarations are not part of ES6, though they are valid in TypeScript:
         *         // static staticProperty;
         *         // property;
         *
         *         constructor(p) { }
         *         static staticMethod(p) { }
         *         method(p) { }
         *     }
         *
         *     // constructor
         *     result = Reflect.deleteMetadata("custom:annotation", Example);
         *
         *     // property (on constructor)
         *     result = Reflect.deleteMetadata("custom:annotation", Example, "staticProperty");
         *
         *     // property (on prototype)
         *     result = Reflect.deleteMetadata("custom:annotation", Example.prototype, "property");
         *
         *     // method (on constructor)
         *     result = Reflect.deleteMetadata("custom:annotation", Example, "staticMethod");
         *
         *     // method (on prototype)
         *     result = Reflect.deleteMetadata("custom:annotation", Example.prototype, "method");
         *
         */
        function deleteMetadata(metadataKey, target, propertyKey) {
            if (!IsObject(target))
                throw new TypeError();
            if (!IsUndefined(propertyKey))
                propertyKey = ToPropertyKey(propertyKey);
            var metadataMap = GetOrCreateMetadataMap(target, propertyKey, /*Create*/ false);
            if (IsUndefined(metadataMap))
                return false;
            if (!metadataMap.delete(metadataKey))
                return false;
            if (metadataMap.size > 0)
                return true;
            var targetMetadata = Metadata.get(target);
            targetMetadata.delete(propertyKey);
            if (targetMetadata.size > 0)
                return true;
            Metadata.delete(target);
            return true;
        }
        exporter("deleteMetadata", deleteMetadata);
        function DecorateConstructor(decorators, target) {
            for (var i = decorators.length - 1; i >= 0; --i) {
                var decorator = decorators[i];
                var decorated = decorator(target);
                if (!IsUndefined(decorated) && !IsNull(decorated)) {
                    if (!IsConstructor(decorated))
                        throw new TypeError();
                    target = decorated;
                }
            }
            return target;
        }
        function DecorateProperty(decorators, target, propertyKey, descriptor) {
            for (var i = decorators.length - 1; i >= 0; --i) {
                var decorator = decorators[i];
                var decorated = decorator(target, propertyKey, descriptor);
                if (!IsUndefined(decorated) && !IsNull(decorated)) {
                    if (!IsObject(decorated))
                        throw new TypeError();
                    descriptor = decorated;
                }
            }
            return descriptor;
        }
        function GetOrCreateMetadataMap(O, P, Create) {
            var targetMetadata = Metadata.get(O);
            if (IsUndefined(targetMetadata)) {
                if (!Create)
                    return undefined;
                targetMetadata = new _Map();
                Metadata.set(O, targetMetadata);
            }
            var metadataMap = targetMetadata.get(P);
            if (IsUndefined(metadataMap)) {
                if (!Create)
                    return undefined;
                metadataMap = new _Map();
                targetMetadata.set(P, metadataMap);
            }
            return metadataMap;
        }
        // 3.1.1.1 OrdinaryHasMetadata(MetadataKey, O, P)
        // https://rbuckton.github.io/reflect-metadata/#ordinaryhasmetadata
        function OrdinaryHasMetadata(MetadataKey, O, P) {
            var hasOwn = OrdinaryHasOwnMetadata(MetadataKey, O, P);
            if (hasOwn)
                return true;
            var parent = OrdinaryGetPrototypeOf(O);
            if (!IsNull(parent))
                return OrdinaryHasMetadata(MetadataKey, parent, P);
            return false;
        }
        // 3.1.2.1 OrdinaryHasOwnMetadata(MetadataKey, O, P)
        // https://rbuckton.github.io/reflect-metadata/#ordinaryhasownmetadata
        function OrdinaryHasOwnMetadata(MetadataKey, O, P) {
            var metadataMap = GetOrCreateMetadataMap(O, P, /*Create*/ false);
            if (IsUndefined(metadataMap))
                return false;
            return ToBoolean(metadataMap.has(MetadataKey));
        }
        // 3.1.3.1 OrdinaryGetMetadata(MetadataKey, O, P)
        // https://rbuckton.github.io/reflect-metadata/#ordinarygetmetadata
        function OrdinaryGetMetadata(MetadataKey, O, P) {
            var hasOwn = OrdinaryHasOwnMetadata(MetadataKey, O, P);
            if (hasOwn)
                return OrdinaryGetOwnMetadata(MetadataKey, O, P);
            var parent = OrdinaryGetPrototypeOf(O);
            if (!IsNull(parent))
                return OrdinaryGetMetadata(MetadataKey, parent, P);
            return undefined;
        }
        // 3.1.4.1 OrdinaryGetOwnMetadata(MetadataKey, O, P)
        // https://rbuckton.github.io/reflect-metadata/#ordinarygetownmetadata
        function OrdinaryGetOwnMetadata(MetadataKey, O, P) {
            var metadataMap = GetOrCreateMetadataMap(O, P, /*Create*/ false);
            if (IsUndefined(metadataMap))
                return undefined;
            return metadataMap.get(MetadataKey);
        }
        // 3.1.5.1 OrdinaryDefineOwnMetadata(MetadataKey, MetadataValue, O, P)
        // https://rbuckton.github.io/reflect-metadata/#ordinarydefineownmetadata
        function OrdinaryDefineOwnMetadata(MetadataKey, MetadataValue, O, P) {
            var metadataMap = GetOrCreateMetadataMap(O, P, /*Create*/ true);
            metadataMap.set(MetadataKey, MetadataValue);
        }
        // 3.1.6.1 OrdinaryMetadataKeys(O, P)
        // https://rbuckton.github.io/reflect-metadata/#ordinarymetadatakeys
        function OrdinaryMetadataKeys(O, P) {
            var ownKeys = OrdinaryOwnMetadataKeys(O, P);
            var parent = OrdinaryGetPrototypeOf(O);
            if (parent === null)
                return ownKeys;
            var parentKeys = OrdinaryMetadataKeys(parent, P);
            if (parentKeys.length <= 0)
                return ownKeys;
            if (ownKeys.length <= 0)
                return parentKeys;
            var set = new _Set();
            var keys = [];
            for (var _i = 0, ownKeys_1 = ownKeys; _i < ownKeys_1.length; _i++) {
                var key = ownKeys_1[_i];
                var hasKey = set.has(key);
                if (!hasKey) {
                    set.add(key);
                    keys.push(key);
                }
            }
            for (var _a = 0, parentKeys_1 = parentKeys; _a < parentKeys_1.length; _a++) {
                var key = parentKeys_1[_a];
                var hasKey = set.has(key);
                if (!hasKey) {
                    set.add(key);
                    keys.push(key);
                }
            }
            return keys;
        }
        // 3.1.7.1 OrdinaryOwnMetadataKeys(O, P)
        // https://rbuckton.github.io/reflect-metadata/#ordinaryownmetadatakeys
        function OrdinaryOwnMetadataKeys(O, P) {
            var keys = [];
            var metadataMap = GetOrCreateMetadataMap(O, P, /*Create*/ false);
            if (IsUndefined(metadataMap))
                return keys;
            var keysObj = metadataMap.keys();
            var iterator = GetIterator(keysObj);
            var k = 0;
            while (true) {
                var next = IteratorStep(iterator);
                if (!next) {
                    keys.length = k;
                    return keys;
                }
                var nextValue = IteratorValue(next);
                try {
                    keys[k] = nextValue;
                }
                catch (e) {
                    try {
                        IteratorClose(iterator);
                    }
                    finally {
                        throw e;
                    }
                }
                k++;
            }
        }
        // 6 ECMAScript Data Typ0es and Values
        // https://tc39.github.io/ecma262/#sec-ecmascript-data-types-and-values
        function Type(x) {
            if (x === null)
                return 1 /* Null */;
            switch (typeof x) {
                case "undefined": return 0 /* Undefined */;
                case "boolean": return 2 /* Boolean */;
                case "string": return 3 /* String */;
                case "symbol": return 4 /* Symbol */;
                case "number": return 5 /* Number */;
                case "object": return x === null ? 1 /* Null */ : 6 /* Object */;
                default: return 6 /* Object */;
            }
        }
        // 6.1.1 The Undefined Type
        // https://tc39.github.io/ecma262/#sec-ecmascript-language-types-undefined-type
        function IsUndefined(x) {
            return x === undefined;
        }
        // 6.1.2 The Null Type
        // https://tc39.github.io/ecma262/#sec-ecmascript-language-types-null-type
        function IsNull(x) {
            return x === null;
        }
        // 6.1.5 The Symbol Type
        // https://tc39.github.io/ecma262/#sec-ecmascript-language-types-symbol-type
        function IsSymbol(x) {
            return typeof x === "symbol";
        }
        // 6.1.7 The Object Type
        // https://tc39.github.io/ecma262/#sec-object-type
        function IsObject(x) {
            return typeof x === "object" ? x !== null : typeof x === "function";
        }
        // 7.1 Type Conversion
        // https://tc39.github.io/ecma262/#sec-type-conversion
        // 7.1.1 ToPrimitive(input [, PreferredType])
        // https://tc39.github.io/ecma262/#sec-toprimitive
        function ToPrimitive(input, PreferredType) {
            switch (Type(input)) {
                case 0 /* Undefined */: return input;
                case 1 /* Null */: return input;
                case 2 /* Boolean */: return input;
                case 3 /* String */: return input;
                case 4 /* Symbol */: return input;
                case 5 /* Number */: return input;
            }
            var hint = PreferredType === 3 /* String */ ? "string" : PreferredType === 5 /* Number */ ? "number" : "default";
            var exoticToPrim = GetMethod(input, toPrimitiveSymbol);
            if (exoticToPrim !== undefined) {
                var result = exoticToPrim.call(input, hint);
                if (IsObject(result))
                    throw new TypeError();
                return result;
            }
            return OrdinaryToPrimitive(input, hint === "default" ? "number" : hint);
        }
        // 7.1.1.1 OrdinaryToPrimitive(O, hint)
        // https://tc39.github.io/ecma262/#sec-ordinarytoprimitive
        function OrdinaryToPrimitive(O, hint) {
            if (hint === "string") {
                var toString_1 = O.toString;
                if (IsCallable(toString_1)) {
                    var result = toString_1.call(O);
                    if (!IsObject(result))
                        return result;
                }
                var valueOf = O.valueOf;
                if (IsCallable(valueOf)) {
                    var result = valueOf.call(O);
                    if (!IsObject(result))
                        return result;
                }
            }
            else {
                var valueOf = O.valueOf;
                if (IsCallable(valueOf)) {
                    var result = valueOf.call(O);
                    if (!IsObject(result))
                        return result;
                }
                var toString_2 = O.toString;
                if (IsCallable(toString_2)) {
                    var result = toString_2.call(O);
                    if (!IsObject(result))
                        return result;
                }
            }
            throw new TypeError();
        }
        // 7.1.2 ToBoolean(argument)
        // https://tc39.github.io/ecma262/2016/#sec-toboolean
        function ToBoolean(argument) {
            return !!argument;
        }
        // 7.1.12 ToString(argument)
        // https://tc39.github.io/ecma262/#sec-tostring
        function ToString(argument) {
            return "" + argument;
        }
        // 7.1.14 ToPropertyKey(argument)
        // https://tc39.github.io/ecma262/#sec-topropertykey
        function ToPropertyKey(argument) {
            var key = ToPrimitive(argument, 3 /* String */);
            if (IsSymbol(key))
                return key;
            return ToString(key);
        }
        // 7.2 Testing and Comparison Operations
        // https://tc39.github.io/ecma262/#sec-testing-and-comparison-operations
        // 7.2.2 IsArray(argument)
        // https://tc39.github.io/ecma262/#sec-isarray
        function IsArray(argument) {
            return Array.isArray
                ? Array.isArray(argument)
                : argument instanceof Object
                    ? argument instanceof Array
                    : Object.prototype.toString.call(argument) === "[object Array]";
        }
        // 7.2.3 IsCallable(argument)
        // https://tc39.github.io/ecma262/#sec-iscallable
        function IsCallable(argument) {
            // NOTE: This is an approximation as we cannot check for [[Call]] internal method.
            return typeof argument === "function";
        }
        // 7.2.4 IsConstructor(argument)
        // https://tc39.github.io/ecma262/#sec-isconstructor
        function IsConstructor(argument) {
            // NOTE: This is an approximation as we cannot check for [[Construct]] internal method.
            return typeof argument === "function";
        }
        // 7.2.7 IsPropertyKey(argument)
        // https://tc39.github.io/ecma262/#sec-ispropertykey
        function IsPropertyKey(argument) {
            switch (Type(argument)) {
                case 3 /* String */: return true;
                case 4 /* Symbol */: return true;
                default: return false;
            }
        }
        // 7.3 Operations on Objects
        // https://tc39.github.io/ecma262/#sec-operations-on-objects
        // 7.3.9 GetMethod(V, P)
        // https://tc39.github.io/ecma262/#sec-getmethod
        function GetMethod(V, P) {
            var func = V[P];
            if (func === undefined || func === null)
                return undefined;
            if (!IsCallable(func))
                throw new TypeError();
            return func;
        }
        // 7.4 Operations on Iterator Objects
        // https://tc39.github.io/ecma262/#sec-operations-on-iterator-objects
        function GetIterator(obj) {
            var method = GetMethod(obj, iteratorSymbol);
            if (!IsCallable(method))
                throw new TypeError(); // from Call
            var iterator = method.call(obj);
            if (!IsObject(iterator))
                throw new TypeError();
            return iterator;
        }
        // 7.4.4 IteratorValue(iterResult)
        // https://tc39.github.io/ecma262/2016/#sec-iteratorvalue
        function IteratorValue(iterResult) {
            return iterResult.value;
        }
        // 7.4.5 IteratorStep(iterator)
        // https://tc39.github.io/ecma262/#sec-iteratorstep
        function IteratorStep(iterator) {
            var result = iterator.next();
            return result.done ? false : result;
        }
        // 7.4.6 IteratorClose(iterator, completion)
        // https://tc39.github.io/ecma262/#sec-iteratorclose
        function IteratorClose(iterator) {
            var f = iterator["return"];
            if (f)
                f.call(iterator);
        }
        // 9.1 Ordinary Object Internal Methods and Internal Slots
        // https://tc39.github.io/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots
        // 9.1.1.1 OrdinaryGetPrototypeOf(O)
        // https://tc39.github.io/ecma262/#sec-ordinarygetprototypeof
        function OrdinaryGetPrototypeOf(O) {
            var proto = Object.getPrototypeOf(O);
            if (typeof O !== "function" || O === functionPrototype)
                return proto;
            // TypeScript doesn't set __proto__ in ES5, as it's non-standard.
            // Try to determine the superclass constructor. Compatible implementations
            // must either set __proto__ on a subclass constructor to the superclass constructor,
            // or ensure each class has a valid `constructor` property on its prototype that
            // points back to the constructor.
            // If this is not the same as Function.[[Prototype]], then this is definately inherited.
            // This is the case when in ES6 or when using __proto__ in a compatible browser.
            if (proto !== functionPrototype)
                return proto;
            // If the super prototype is Object.prototype, null, or undefined, then we cannot determine the heritage.
            var prototype = O.prototype;
            var prototypeProto = prototype && Object.getPrototypeOf(prototype);
            if (prototypeProto == null || prototypeProto === Object.prototype)
                return proto;
            // If the constructor was not a function, then we cannot determine the heritage.
            var constructor = prototypeProto.constructor;
            if (typeof constructor !== "function")
                return proto;
            // If we have some kind of self-reference, then we cannot determine the heritage.
            if (constructor === O)
                return proto;
            // we have a pretty good guess at the heritage.
            return constructor;
        }
        // naive Map shim
        function CreateMapPolyfill() {
            var cacheSentinel = {};
            var arraySentinel = [];
            var MapIterator = (function () {
                function MapIterator(keys, values, selector) {
                    this._index = 0;
                    this._keys = keys;
                    this._values = values;
                    this._selector = selector;
                }
                MapIterator.prototype["@@iterator"] = function () { return this; };
                MapIterator.prototype[iteratorSymbol] = function () { return this; };
                MapIterator.prototype.next = function () {
                    var index = this._index;
                    if (index >= 0 && index < this._keys.length) {
                        var result = this._selector(this._keys[index], this._values[index]);
                        if (index + 1 >= this._keys.length) {
                            this._index = -1;
                            this._keys = arraySentinel;
                            this._values = arraySentinel;
                        }
                        else {
                            this._index++;
                        }
                        return { value: result, done: false };
                    }
                    return { value: undefined, done: true };
                };
                MapIterator.prototype.throw = function (error) {
                    if (this._index >= 0) {
                        this._index = -1;
                        this._keys = arraySentinel;
                        this._values = arraySentinel;
                    }
                    throw error;
                };
                MapIterator.prototype.return = function (value) {
                    if (this._index >= 0) {
                        this._index = -1;
                        this._keys = arraySentinel;
                        this._values = arraySentinel;
                    }
                    return { value: value, done: true };
                };
                return MapIterator;
            }());
            return (function () {
                function Map() {
                    this._keys = [];
                    this._values = [];
                    this._cacheKey = cacheSentinel;
                    this._cacheIndex = -2;
                }
                Object.defineProperty(Map.prototype, "size", {
                    get: function () { return this._keys.length; },
                    enumerable: true,
                    configurable: true
                });
                Map.prototype.has = function (key) { return this._find(key, /*insert*/ false) >= 0; };
                Map.prototype.get = function (key) {
                    var index = this._find(key, /*insert*/ false);
                    return index >= 0 ? this._values[index] : undefined;
                };
                Map.prototype.set = function (key, value) {
                    var index = this._find(key, /*insert*/ true);
                    this._values[index] = value;
                    return this;
                };
                Map.prototype.delete = function (key) {
                    var index = this._find(key, /*insert*/ false);
                    if (index >= 0) {
                        var size = this._keys.length;
                        for (var i = index + 1; i < size; i++) {
                            this._keys[i - 1] = this._keys[i];
                            this._values[i - 1] = this._values[i];
                        }
                        this._keys.length--;
                        this._values.length--;
                        if (key === this._cacheKey) {
                            this._cacheKey = cacheSentinel;
                            this._cacheIndex = -2;
                        }
                        return true;
                    }
                    return false;
                };
                Map.prototype.clear = function () {
                    this._keys.length = 0;
                    this._values.length = 0;
                    this._cacheKey = cacheSentinel;
                    this._cacheIndex = -2;
                };
                Map.prototype.keys = function () { return new MapIterator(this._keys, this._values, getKey); };
                Map.prototype.values = function () { return new MapIterator(this._keys, this._values, getValue); };
                Map.prototype.entries = function () { return new MapIterator(this._keys, this._values, getEntry); };
                Map.prototype["@@iterator"] = function () { return this.entries(); };
                Map.prototype[iteratorSymbol] = function () { return this.entries(); };
                Map.prototype._find = function (key, insert) {
                    if (this._cacheKey !== key) {
                        this._cacheIndex = this._keys.indexOf(this._cacheKey = key);
                    }
                    if (this._cacheIndex < 0 && insert) {
                        this._cacheIndex = this._keys.length;
                        this._keys.push(key);
                        this._values.push(undefined);
                    }
                    return this._cacheIndex;
                };
                return Map;
            }());
            function getKey(key, _) {
                return key;
            }
            function getValue(_, value) {
                return value;
            }
            function getEntry(key, value) {
                return [key, value];
            }
        }
        // naive Set shim
        function CreateSetPolyfill() {
            return (function () {
                function Set() {
                    this._map = new _Map();
                }
                Object.defineProperty(Set.prototype, "size", {
                    get: function () { return this._map.size; },
                    enumerable: true,
                    configurable: true
                });
                Set.prototype.has = function (value) { return this._map.has(value); };
                Set.prototype.add = function (value) { return this._map.set(value, value), this; };
                Set.prototype.delete = function (value) { return this._map.delete(value); };
                Set.prototype.clear = function () { this._map.clear(); };
                Set.prototype.keys = function () { return this._map.keys(); };
                Set.prototype.values = function () { return this._map.values(); };
                Set.prototype.entries = function () { return this._map.entries(); };
                Set.prototype["@@iterator"] = function () { return this.keys(); };
                Set.prototype[iteratorSymbol] = function () { return this.keys(); };
                return Set;
            }());
        }
        // naive WeakMap shim
        function CreateWeakMapPolyfill() {
            var UUID_SIZE = 16;
            var keys = HashMap.create();
            var rootKey = CreateUniqueKey();
            return (function () {
                function WeakMap() {
                    this._key = CreateUniqueKey();
                }
                WeakMap.prototype.has = function (target) {
                    var table = GetOrCreateWeakMapTable(target, /*create*/ false);
                    return table !== undefined ? HashMap.has(table, this._key) : false;
                };
                WeakMap.prototype.get = function (target) {
                    var table = GetOrCreateWeakMapTable(target, /*create*/ false);
                    return table !== undefined ? HashMap.get(table, this._key) : undefined;
                };
                WeakMap.prototype.set = function (target, value) {
                    var table = GetOrCreateWeakMapTable(target, /*create*/ true);
                    table[this._key] = value;
                    return this;
                };
                WeakMap.prototype.delete = function (target) {
                    var table = GetOrCreateWeakMapTable(target, /*create*/ false);
                    return table !== undefined ? delete table[this._key] : false;
                };
                WeakMap.prototype.clear = function () {
                    // NOTE: not a real clear, just makes the previous data unreachable
                    this._key = CreateUniqueKey();
                };
                return WeakMap;
            }());
            function CreateUniqueKey() {
                var key;
                do
                    key = "@@WeakMap@@" + CreateUUID();
                while (HashMap.has(keys, key));
                keys[key] = true;
                return key;
            }
            function GetOrCreateWeakMapTable(target, create) {
                if (!hasOwn.call(target, rootKey)) {
                    if (!create)
                        return undefined;
                    Object.defineProperty(target, rootKey, { value: HashMap.create() });
                }
                return target[rootKey];
            }
            function FillRandomBytes(buffer, size) {
                for (var i = 0; i < size; ++i)
                    buffer[i] = Math.random() * 0xff | 0;
                return buffer;
            }
            function GenRandomBytes(size) {
                if (typeof Uint8Array === "function") {
                    if (typeof crypto !== "undefined")
                        return crypto.getRandomValues(new Uint8Array(size));
                    if (typeof msCrypto !== "undefined")
                        return msCrypto.getRandomValues(new Uint8Array(size));
                    return FillRandomBytes(new Uint8Array(size), size);
                }
                return FillRandomBytes(new Array(size), size);
            }
            function CreateUUID() {
                var data = GenRandomBytes(UUID_SIZE);
                // mark as random - RFC 4122 § 4.4
                data[6] = data[6] & 0x4f | 0x40;
                data[8] = data[8] & 0xbf | 0x80;
                var result = "";
                for (var offset = 0; offset < UUID_SIZE; ++offset) {
                    var byte = data[offset];
                    if (offset === 4 || offset === 6 || offset === 8)
                        result += "-";
                    if (byte < 16)
                        result += "0";
                    result += byte.toString(16).toLowerCase();
                }
                return result;
            }
        }
        // uses a heuristic used by v8 and chakra to force an object into dictionary mode.
        function MakeDictionary(obj) {
            obj.__ = undefined;
            delete obj.__;
            return obj;
        }
    });
})(Reflect$1 || (Reflect$1 = {}));

/**
 * Decorators for storing meta data on types and properties
 * for runtime type checking, schema validation
 * and serialization / serialization.
 */
/**
 * Define a type.
 *
 * @param id The `@id` of the type e.g. `schema:Thing` for https://schema.org/Thing
 */
function type(id) {
    return function (target) {
        Reflect.defineMetadata('type:id', id, target);
    };
}
/**
 * Define a property.
 *
 * @param id        The `@id` of the property e.g. `schema:name` for https://schema.org/name
 * @param container The `@container` type for the property. Must be `list` or `set` (default).
 *                  A `list` is a ordered collection. A `set` is an unordered collection.
 *                  See the [JSON-LD docs](https://w3c.github.io/json-ld-syntax/#sets-and-lists)
 *                  for more info.
 */
function property(id, container = 'set') {
    return function (target, propertyKey) {
        Reflect.defineMetadata('property:id', id, target, propertyKey);
        if (container)
            Reflect.defineMetadata('property:container', container, target, propertyKey);
    };
}

/**
 * The most generic type of item.
 *
 * This is base class for all other classes in this schema.
 * As well as definining the properties of a `schema:Thing` it
 * implements methods such as `toJSONLD` for marshalling to JSON-LD.
 *
 * @see {@link https://schema.org/Thing}
 */
let Thing = class Thing {
    /**
     * Constructor
     *
     * Uses the values of properties in the initializer.
     * Only registered properties (i.e. those with the @property decorator) are initialized.
     * All other values are ignored without warning.
     *
     * @param initializer An object with initial property values
     */
    constructor(initializer = {}) {
        /**
         * The JSON-LD [node identifier](https://w3c.github.io/json-ld-syntax/#node-identifiers) corresponding to
         * the `@id` keyword.
         */
        this.id = '';
        /**
         * A description of the item.
         *
         * @see {@link https://schema.org/description}
         */
        this.description = '';
        /**
         * The identifier property represents any kind of identifier for any kind of Thing,
         * such as ISBNs, GTIN codes, UUIDs etc. Schema.org provides dedicated properties
         * for representing many of these, either as textual strings or as URL (URI) links.
         *
         * @see {@link https://schema.org/identifier}
         */
        this.identifiers = [];
        /**
         * The name of the item.
         *
         * @see {@link https://schema.org/name}
         */
        this.name = '';
        /**
         * URL of the item.
         *
         * @see {@link https://schema.org/url}
         */
        this.urls = [];
        for (let [key, value] of Object.entries(initializer)) {
            if (Reflect.hasMetadata('property:id', this, key)) {
                // @ts-ignore
                this[key] = value;
            }
        }
    }
    /**
     * The JSON-LD [type specifier](https://w3c.github.io/json-ld-syntax/#specifying-the-type) corresponding to
     * the `@type` keyword.
     */
    get type() {
        return this.constructor.name;
    }
};
__decorate([
    property('schema:description'),
    __metadata("design:type", String)
], Thing.prototype, "description", void 0);
__decorate([
    property('schema:identifier'),
    __metadata("design:type", Array)
], Thing.prototype, "identifiers", void 0);
__decorate([
    property('schema:name'),
    __metadata("design:type", String)
], Thing.prototype, "name", void 0);
__decorate([
    property('schema:url'),
    __metadata("design:type", Array)
], Thing.prototype, "urls", void 0);
Thing = __decorate([
    type('schema:Thing'),
    __metadata("design:paramtypes", [Object])
], Thing);
var Thing$1 = Thing;

/**
 * A utility class that serves as the umbrella for a number of 'intangible'
 * things such as quantities, structured values, etc.
 *
 * @see {@link https://schema.org/Intangible}
 */
let Intangible = class Intangible extends Thing$1 {
};
Intangible = __decorate([
    type('schema:Intangible')
], Intangible);
var Intangible$1 = Intangible;

var ComputerLanguage_1;
/**
 * This type covers computer programming languages such as Scheme and Lisp,
 * as well as other language-like computer representations.
 * Natural languages are best represented with the Language type.
 *
 * @see {@link https://schema.org/ComputerLanguage}
 */
let ComputerLanguage = ComputerLanguage_1 = class ComputerLanguage extends Intangible$1 {
};
// Instances of computer languages
/**
 * Javascript programming language
 *
 * @see {@link https://www.wikidata.org/wiki/Q2005}
 */
ComputerLanguage.js = new ComputerLanguage_1({ name: 'JavaScript' });
/**
 * Python general-purpose, high-level programming language
 *
 * @see {@link https://www.wikidata.org/wiki/Q28865}
 */
ComputerLanguage.py = new ComputerLanguage_1({ name: 'Python' });
/**
 * R programming language for statistical computing
 *
 * @see {@link https://www.wikidata.org/wiki/Q206904}
 */
ComputerLanguage.r = new ComputerLanguage_1({ name: 'R' });
ComputerLanguage = ComputerLanguage_1 = __decorate([
    type('schema:ComputerLanguage')
], ComputerLanguage);
var ComputerLanguage$1 = ComputerLanguage;

/**
 * The most generic kind of creative work, including books, movies,
 * photographs, software programs, etc.
 *
 * @see {@link https://schema.org/CreativeWork}
 */
let CreativeWork = class CreativeWork extends Thing$1 {
    /**
     * The most generic kind of creative work, including books, movies,
     * photographs, software programs, etc.
     *
     * @see {@link https://schema.org/CreativeWork}
     */
    constructor() {
        super(...arguments);
        /**
         * The author of this content or rating. Please note that author is special in
         * that HTML 5 provides a special mechanism for indicating authorship via the rel
         * tag. That is equivalent to this and may be used interchangeably.
         *
         * @see {@link https://schema.org/author}
         */
        this.authors = [];
        /**
         * A secondary contributor to the CreativeWork or Event.
         *
         * @see {@link https://schema.org/contributor}
         */
        this.contributors = [];
        /**
         * The creator/author of this CreativeWork. This is the same as
         * the Author property for CreativeWork.
         *
         * @see {@link https://schema.org/creator}
         */
        this.creators = [];
        /**
         * Date of first broadcast/publication.
         *
         * @see {@link https://schema.org/datePublished}
         */
        this.datePublished = '';
        /**
         * Keywords or tags used to describe this content.
         * Multiple entries in a keywords list are typically delimited by commas.
         *
         * @see {@link https://schema.org/keywords}
         */
        this.keywords = '';
        /**
         * A license document that applies to this content, typically indicated by URL.
         *
         * @see {@link https://schema.org/license}
         */
        this.license = '';
        /**
         * The textual content of this CreativeWork.
         *
         * @see {@link https://schema.org/text}
         */
        this.text = '';
        /**
         * The version of the CreativeWork embodied by a specified resource.
         *
         * @see {@link https://schema.org/version}
         */
        this.version = '';
    }
};
__decorate([
    property('schema:author'),
    __metadata("design:type", Array)
], CreativeWork.prototype, "authors", void 0);
__decorate([
    property('schema:contributor'),
    __metadata("design:type", Array)
], CreativeWork.prototype, "contributors", void 0);
__decorate([
    property('schema:creator'),
    __metadata("design:type", Array)
], CreativeWork.prototype, "creators", void 0);
__decorate([
    property('schema:datePublished'),
    __metadata("design:type", String)
], CreativeWork.prototype, "datePublished", void 0);
__decorate([
    property('schema:keywords'),
    __metadata("design:type", String)
], CreativeWork.prototype, "keywords", void 0);
__decorate([
    property('schema:license'),
    __metadata("design:type", Object)
], CreativeWork.prototype, "license", void 0);
__decorate([
    property('schema:text'),
    __metadata("design:type", String)
], CreativeWork.prototype, "text", void 0);
__decorate([
    property('schema:version'),
    __metadata("design:type", Object)
], CreativeWork.prototype, "version", void 0);
CreativeWork = __decorate([
    type('schema:CreativeWork')
], CreativeWork);
var CreativeWork$1 = CreativeWork;

var OperatingSystem_1;
/**
 * A collection of software that manages computer hardware resources
 *
 * @see {@link https://www.wikidata.org/wiki/Q9135}
 */
let OperatingSystem = OperatingSystem_1 = class OperatingSystem extends Intangible$1 {
};
// Instances of OperatingSystem (high level)
/**
 * Linux operating system family that use the Linux kernel. For instance GNU/Linux or Android.
 *
 * @see {@link https://www.wikidata.org/wiki/Q388}
 */
OperatingSystem.linux = new OperatingSystem_1({ name: 'Linux' });
/**
 * macOS operating system for Apple computers, launched in 2001 as Mac OS X
 *
 * @see {@link https://www.wikidata.org/wiki/Q14116}
 */
OperatingSystem.macos = new OperatingSystem_1({ name: 'macOS' });
/**
 * Unix family of computer operating systems that derive from the original AT&T Unix
 *
 * @see {@link https://www.wikidata.org/wiki/Q11368}
 */
OperatingSystem.unix = new OperatingSystem_1({ name: 'Unix' });
/**
 * Windows family of operating systems produced for personal computers,
 * servers, smartphones and embedded devices
 *
 * @see {@link https://www.wikidata.org/wiki/Q1406}
 */
OperatingSystem.windows = new OperatingSystem_1({ name: 'Windows' });
OperatingSystem = OperatingSystem_1 = __decorate([
    type('stencila:OperatingSystem')
], OperatingSystem);
var OperatingSystem$1 = OperatingSystem;

/**
 * An organization such as a school, NGO, corporation, club, etc.
 *
 * @see {@link https://schema.org/Organization}
 */
let Organization = class Organization extends Thing$1 {
};
Organization = __decorate([
    type('schema:Organization')
], Organization);
var Organization$1 = Organization;

var Person_1;
/**
 * A person (alive, dead, undead, or fictional).
 *
 * @see {@link https://schema.org/Person}
 */
let Person = Person_1 = class Person extends Thing$1 {
    /**
     * A person (alive, dead, undead, or fictional).
     *
     * @see {@link https://schema.org/Person}
     */
    constructor() {
        super(...arguments);
        /**
         * A person (alive, dead, undead, or fictional).
         *
         * @see {@link https://schema.org/email}
         */
        this.emails = [];
        /**
         * Family name. In the U.S., the last name of an Person.
         * This can be used along with givenName instead of the name property.
         *
         * @see {@link https://schema.org/familyName}
         */
        this.familyNames = [];
        /**
         * Given name. In the U.S., the first name of a Person.
         * This can be used along with familyName instead of the name property.
         *
         * @see {@link https://schema.org/givenName}
         */
        this.givenNames = [];
    }
    /**
     * Create a `Person` object from a `Text` value.
     *
     * The text value can contain email and URL in the format:
     *
     *   Joe Bloggs <joe@example.com> (https://example.com/joe)
     *
     * @param text The text value to parse
     * @returns A `Person` object
     */
    static fromText(text) {
        const person = new Person_1();
        const match = text.match(/^(?:\s*)([^\s]*)(?:\s+)?([^\s]+)?\s*(<([^>]*)>)?\s*(\(([^)]*)\))?/);
        if (match) {
            if (match[1]) {
                person.givenNames = [match[1]];
                person.name = person.givenNames.join(' ');
            }
            if (match[2]) {
                person.familyNames = [match[2]];
                person.name += ' ' + person.familyNames.join(' ');
            }
            if (match[4])
                person.emails = [match[4]];
            if (match[6])
                person.urls = [match[6]];
        }
        else {
            person.name = text;
        }
        return person;
    }
};
__decorate([
    property('schema:email'),
    __metadata("design:type", Array)
], Person.prototype, "emails", void 0);
__decorate([
    property('schema:familyName'),
    __metadata("design:type", Array)
], Person.prototype, "familyNames", void 0);
__decorate([
    property('schema:givenName'),
    __metadata("design:type", Array)
], Person.prototype, "givenNames", void 0);
Person = Person_1 = __decorate([
    type('schema:Person')
], Person);
var Person$1 = Person;

/**
 * A software application.
 *
 * @see {@link https://schema.org/SoftwareApplication}
 */
let SoftwareApplication = class SoftwareApplication extends CreativeWork$1 {
    /**
     * A software application.
     *
     * @see {@link https://schema.org/SoftwareApplication}
     */
    constructor() {
        super(...arguments);
        /**
         * Type of software application, e.g. 'Game, Multimedia'.
         *
         * @see {@link https://schema.org/applicationCategory}
         */
        this.applicationCategories = [];
        /**
         * Subcategory of the application, e.g. 'Arcade Game'.
         *
         * @see {@link https://schema.org/applicationSubCategory}
         */
        this.applicationSubCategories = [];
        /**
         * Operating systems supported (Windows 7, OSX 10.6, Android 1.6).
         *
         * @see {@link https://schema.org/operatingSystem}
         */
        this.operatingSystems = [];
        /**
         * Component dependency requirements for application.
         * This includes runtime environments and shared libraries that are not included in
         * the application distribution package, but required to run the application.
         *
         * The [`schema:softwareRequirements`](https://schema.org/softwareRequirements)
         * property allows for `Text` or `URL` values. Here, we allow
         * values of software packages or applications.
         *
         * @see {@link https://schema.org/softwareRequirements}
         */
        this.softwareRequirements = [];
    }
};
__decorate([
    property('schema:applicationCategory'),
    __metadata("design:type", Array)
], SoftwareApplication.prototype, "applicationCategories", void 0);
__decorate([
    property('schema:applicationSubCategory'),
    __metadata("design:type", Array)
], SoftwareApplication.prototype, "applicationSubCategories", void 0);
__decorate([
    property('schema:operatingSystem'),
    __metadata("design:type", Array)
], SoftwareApplication.prototype, "operatingSystems", void 0);
__decorate([
    property('schema:softwareRequirements'),
    __metadata("design:type", Array)
], SoftwareApplication.prototype, "softwareRequirements", void 0);
SoftwareApplication = __decorate([
    type('schema:SoftwareApplication')
], SoftwareApplication);
var SoftwareApplication$1 = SoftwareApplication;

/**
 * A software environment
 *
 * Currently only used in [Dockter](https://github.com/stencila/dockter)
 * form which a Dockerfile is generated.
 *
 * This may be replaced by `openschemas:Container`.
 * See https://github.com/stencila/schema/issues/11
 */
let SoftwareEnvironment = class SoftwareEnvironment extends SoftwareApplication$1 {
};
SoftwareEnvironment = __decorate([
    type('stencila:SoftwareEnvironment')
], SoftwareEnvironment);
var SoftwareEnvironment$1 = SoftwareEnvironment;

/**
 * Computer programming source code. Example: Full (compile ready) solutions,
 * code snippet samples, scripts, templates.
 *
 * @see {@link https://schema.org/SoftwareSourceCode}
 */
let SoftwareSourceCode = class SoftwareSourceCode extends CreativeWork$1 {
    /**
     * Computer programming source code. Example: Full (compile ready) solutions,
     * code snippet samples, scripts, templates.
     *
     * @see {@link https://schema.org/SoftwareSourceCode}
     */
    constructor() {
        super(...arguments);
        /**
         * Link to the repository where the un-compiled, human readable code and
         * related code is located (SVN, github, CodePlex).
         *
         * @see {@link https://schema.org/codeRepository}
         */
        this.codeRepository = '';
        /**
         * What type of code sample: full (compile ready) solution, code snippet,
         * inline code, scripts, template.
         *
         * @see {@link https://schema.org/codeSampleType}
         */
        this.codeSampleType = '';
        /**
         * Individual responsible for maintaining the software (usually includes an email contact address).
         *
         * Note that CodeMeta says that `maintainer` should be a `Person`, not `Organization` or `Person`
         * as with `author`
         *
         * @see {@link https://codemeta.github.io/terms/}
         */
        this.maintainers = [];
        /**
         * The computer programming language.
         *
         * @see {@link https://schema.org/programmingLanguage}
         */
        this.programmingLanguages = [];
        /**
         * Runtime platform or script interpreter dependencies (Example - Java v1,
         * Python2.3, .Net Framework 3.0).
         *
         * @see {@link https://schema.org/runtimePlatform}
         */
        this.runtimePlatform = '';
        /**
         * Target Operating System / Product to which the code applies. If applies to
         * several versions, just the product name can be used.
         *
         * @see {@link https://schema.org/targetProduct}
         */
        this.targetProducts = [];
    }
};
__decorate([
    property('schema:codeRepository'),
    __metadata("design:type", String)
], SoftwareSourceCode.prototype, "codeRepository", void 0);
__decorate([
    property('schema:codeSampleType'),
    __metadata("design:type", String)
], SoftwareSourceCode.prototype, "codeSampleType", void 0);
__decorate([
    property('codemeta:maintainer'),
    __metadata("design:type", Array)
], SoftwareSourceCode.prototype, "maintainers", void 0);
__decorate([
    property('schema:programmingLanguage'),
    __metadata("design:type", Array)
], SoftwareSourceCode.prototype, "programmingLanguages", void 0);
__decorate([
    property('schema:runtimePlatform'),
    __metadata("design:type", String)
], SoftwareSourceCode.prototype, "runtimePlatform", void 0);
__decorate([
    property('schema:targetProduct'),
    __metadata("design:type", Array)
], SoftwareSourceCode.prototype, "targetProducts", void 0);
SoftwareSourceCode = __decorate([
    type('schema:SoftwareSourceCode')
], SoftwareSourceCode);
var SoftwareSourceCode$1 = SoftwareSourceCode;

/**
 * A software package.
 *
 * This is an extension class defined for this context.
 * It is necessary because `schema:SoftwareSourceCode`
 * has most, but not all, of the properties that we need to represent a package,
 * for applications such as Dockter.
 * Meanwhile, `schema:SoftwareApplication` has some of those missing
 * properties but lacks most. This type does
 * not introduce any new properties, but rather uses
 * schema.org properties on a subtype of `schema:SoftwareSourceCode`
 *
 * An alternative approach would be to create a `SoftwareApplication` which
 * links to one or more `SoftwarePackages`. See https://github.com/codemeta/codemeta/issues/198
 */
let SoftwarePackage = class SoftwarePackage extends SoftwareSourceCode$1 {
    /**
     * A software package.
     *
     * This is an extension class defined for this context.
     * It is necessary because `schema:SoftwareSourceCode`
     * has most, but not all, of the properties that we need to represent a package,
     * for applications such as Dockter.
     * Meanwhile, `schema:SoftwareApplication` has some of those missing
     * properties but lacks most. This type does
     * not introduce any new properties, but rather uses
     * schema.org properties on a subtype of `schema:SoftwareSourceCode`
     *
     * An alternative approach would be to create a `SoftwareApplication` which
     * links to one or more `SoftwarePackages`. See https://github.com/codemeta/codemeta/issues/198
     */
    constructor() {
        super(...arguments);
        /**
         * Type of software application, e.g. 'Game, Multimedia'.
         *
         * @see {@link https://schema.org/applicationCategory}
         */
        this.applicationCategories = [];
        /**
         * Subcategory of the application, e.g. 'Arcade Game'.
         *
         * @see {@link https://schema.org/applicationSubCategory}
         */
        this.applicationSubCategories = [];
        /**
         * Operating systems supported (Windows 7, OSX 10.6, Android 1.6).
         *
         * `schema:operatingSystem` expects type `Text`, whereas here we
         * expect `OperatingSystem`
         *
         * @see {@link https://schema.org/operatingSystem}
         */
        this.operatingSystems = [];
        /**
         * Component dependency requirements for application.
         * This includes runtime environments and shared libraries that are not included in
         * the application distribution package, but required to run the application.
         *
         * The [`schema:softwareRequirements`](https://schema.org/softwareRequirements)
         * property allows for `Text` or `URL` values. Here, we allow
         * values of software packages or applications.
         *
         * @see {@link https://schema.org/softwareRequirements}
         */
        this.softwareRequirements = [];
    }
};
__decorate([
    property('schema:applicationCategory'),
    __metadata("design:type", Array)
], SoftwarePackage.prototype, "applicationCategories", void 0);
__decorate([
    property('schema:applicationSubCategory'),
    __metadata("design:type", Array)
], SoftwarePackage.prototype, "applicationSubCategories", void 0);
__decorate([
    property('schema:operatingSystem'),
    __metadata("design:type", Array)
], SoftwarePackage.prototype, "operatingSystems", void 0);
__decorate([
    property('schema:softwareRequirements'),
    __metadata("design:type", Array)
], SoftwarePackage.prototype, "softwareRequirements", void 0);
SoftwarePackage = __decorate([
    type('stencila:SoftwarePackage')
], SoftwarePackage);
var SoftwarePackage$1 = SoftwarePackage;

/**
 * Represents a session within a `SoftwareEnvironment`
 *
 * We may be able to use `openschemas::Container` instead.
 */
let SoftwareSession = class SoftwareSession extends Thing$1 {
};
SoftwareSession = __decorate([
    type('stencila:SoftwareSession')
], SoftwareSession);
var SoftwareSession$1 = SoftwareSession;



var types = /*#__PURE__*/Object.freeze({
	ComputerLanguage: ComputerLanguage$1,
	CreativeWork: CreativeWork$1,
	Intangible: Intangible$1,
	OperatingSystem: OperatingSystem$1,
	Organization: Organization$1,
	Person: Person$1,
	SoftwareApplication: SoftwareApplication$1,
	SoftwareEnvironment: SoftwareEnvironment$1,
	SoftwarePackage: SoftwarePackage$1,
	SoftwareSession: SoftwareSession$1,
	SoftwareSourceCode: SoftwareSourceCode$1,
	Thing: Thing$1
});

// The was `const pkg = require('../package')` but since that
// doesn't work in the browser this is a temporrary workaround
const pkg = {
    name: '@stencila/schema',
    version: '0.0.0',
    homepage: 'https://stencila.github.io/schema/'
};
class Processor {
    /**
     * Get the manifest for this processor
     */
    manifest() {
        return {
            stencila: {
                name: pkg.name,
                url: pkg.homepage,
                version: pkg.version
            },
            services: {
                import: ['application/ld+json'],
                export: ['application/ld+json'],
                compile: [],
                build: [],
                execute: []
            }
        };
    }
    /**
     * Import a `Thing`.
     *
     * @param thing The thing to be imported
     * @param format The current format of the thing as a MIME type e.g. `text/markdown`
     * @returns An instance of a class derived from `Thing`
     */
    import(thing, format = 'application/ld+json') {
        if (thing instanceof Thing$1) {
            return thing;
        }
        else if (typeof thing === 'object') {
            return this.importObject(thing);
        }
        else {
            switch (format) {
                case 'application/ld+json':
                    return this.importJsonLd(thing);
                default:
                    throw Error(`Unhandled import format: ${format}`);
            }
        }
    }
    /**
     * Import an `Object` to a `Thing`
     *
     * This function demarshalls a plain JavaScript object into an
     * instance of a class derived from `Thing` based on the `type`
     * property of the object.
     *
     * @param object A plain JavaScript object with a `type` property
     * @returns An instance of a class derived from `Thing`
     */
    importObject(object) {
        const type = object.type;
        if (!type)
            throw new Error('Object is missing required "type" property');
        // @ts-ignore
        const Type = types[type];
        if (!Type)
            throw new Error(`Unknown type "${type}"`);
        return new Type(object);
    }
    /**
     * Import a JSON-LD document to a `Thing`
     *
     * @param jsonld A JSON-LD document with a `type` property
     * @returns An instance of a class derived from `Thing`
     */
    importJsonLd(jsonld) {
        const object = JSON.parse(jsonld);
        return this.importObject(object);
    }
    /**
     * Export a `Thing`.
     *
     * @param thing The thing to be exported
     * @param format The format, as a MIME type, to export to e.g. `text/html`
     */
    export(thing, format = 'application/ld+json') {
        if (!(thing instanceof Thing$1))
            thing = this.import(thing);
        switch (format) {
            case 'application/ld+json':
                return this.exportJsonLd(thing);
            default:
                throw Error(`Unhandled export format: ${format}`);
        }
    }
    /**
     * Export a `Thing` to a JSON-LD string
     *
     * @param thing The thing to be exported
     */
    exportJsonLd(thing) {
        const obj = Object.assign({
            '@context': 'https://stencila.github.io/schema/context.jsonld'
        }, this.exportObject(thing));
        return JSON.stringify(obj);
    }
    /**
     * Export a `Thing` to an `Object`
     *
     * This function marshalls a `Thing` to a plain JavaScript object
     * having a `type` and other properties of the type of thing.
     *
     * @param thing The thing to be exported
     */
    exportObject(thing) {
        const obj = {};
        obj['type'] = thing.type;
        for (let [key, value] of Object.entries(thing)) {
            if (typeof value === 'string' && value.length === 0)
                continue;
            if (Array.isArray(value) && value.length === 0)
                continue;
            let id = Reflect.getMetadata('property:id', thing, key);
            let [context, term] = id.split(':');
            if (Array.isArray(value)) {
                obj[term] = value.map(item => (item instanceof Thing$1) ? this.exportObject(item) : item);
            }
            else if (value instanceof Thing$1) {
                obj[term] = this.exportObject(value);
            }
            else {
                obj[term] = value;
            }
        }
        return obj;
    }
    /**
     * Convert a thing from one format to another.
     *
     * @param thing The thing to convert as a string
     * @param from The current format of the thing as a MIME type e.g. `text/markdown`
     * @param to The desired format for the thing as a MIME type e.g. `text/html`
     */
    convert(thing, from = 'application/ld+json', to = 'application/ld+json') {
        return this.export(this.import(thing, from), to);
    }
    /**
     * Compile a thing
     *
     * @param thing The thing to compile
     * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
     */
    compile(thing, format = 'application/ld+json') {
        thing = this.import(thing, format);
        return thing;
    }
    /**
     * Build a `Thing`.
     *
     * The `build` function, like the `compile` function is used to prepare a thing
     * for execution. However, it usually involves the creation of build artifacts
     * (which may take some time to build) that are exernal to the thing
     * e.g. a binary executable or Docker image.
     * Like `compile`, it may add or modify properties of the thing
     * such as providing a URL to the built artifacts.
     *
     * @param thing The thing to build
     * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
     */
    build(thing, format = 'application/ld+json') {
        thing = this.compile(thing, format);
        return thing;
    }
    /**
     * Execute a thing
     *
     * @param thing The thing to execute
     * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
     */
    execute(thing, format = 'application/ld+json') {
        thing = this.build(thing, format);
        return thing;
    }
}

/**
 * An instance of the base `Processor` class used to import
 * the result of remove calls as a `Thing`.
 */
const processor = new Processor();
/**
 * A base client class which acts as a proxy to a remote `Processor`.
 * Implements aynchronous, proxy methods for `Processor` methods `compile`, `execute`, etc.
 * Those methods send JSON-RPC requests to a `Server` that is serving the remote processor.
 */
class Client {
    constructor() {
        /**
         * A map of requests to which responses can be paired against
         */
        this.requests = {};
    }
    /**
     * Proxy to the remote `Processor`'s `manifest` method
     */
    async manifest() {
        return this.call('manifest');
    }
    /**
     * Proxy to the remote `Processor`'s `import` method
     */
    async import(thing, format = 'application/ld+json') {
        return this.call('import', thing, format);
    }
    /**
     * Proxy to the remote `Processor`'s `export` method
     */
    async export(thing, format = 'application/ld+json') {
        return this.call('export', thing, format);
    }
    /**
     * Proxy to the remote `Processor`'s `convert` method
     */
    async convert(thing, from = 'application/ld+json', to = 'application/ld+json') {
        return this.call('convert', thing, from, to);
    }
    /**
     * Proxy to the remote `Processor`'s `compile` method
     */
    async compile(thing, format = 'application/ld+json') {
        return this.call('compile', thing, format);
    }
    /**
     * Proxy to the remote `Processor`'s `build` method
     */
    async build(thing, format = 'application/ld+json') {
        return this.call('build', thing, format);
    }
    /**
     * Proxy to the remote `Processor`'s `execute` method
     */
    async execute(thing, format = 'application/ld+json') {
        return this.call('execute', thing, format);
    }
    /**
     * Call a method of a remote `Processor`.
     *
     * @param method The name of the method
     * @param args Any method arguments
     */
    async call(method, ...args) {
        // Create a new request
        for (let index in args) {
            const arg = args[index];
            if (typeof arg === 'object' && arg.type) {
                args[index] = Object.assign({}, arg, { type: arg.type });
            }
        }
        const request = new JsonRpcRequest(method, args);
        // ... and a promise that resolves to the resul
        const promise = new Promise((resolve, reject) => {
            this.requests[request.id] = (response) => {
                // Fails if the response has an error
                if (response.error)
                    return reject(new Error(response.error.message));
                // If the result is a `Thing` then we want to import it
                const result = response.result;
                if (typeof result === 'object' && result.type) {
                    const thing = processor.importObject(response.result);
                    // @ts-ignore
                    resolve(thing);
                }
                else {
                    resolve(result);
                }
            };
        });
        // Now, we're ready to send the request
        this.send(request);
        return promise;
    }
    /**
     * Receive a request from the server.
     *
     * Uses the `id` of the response to match it to the corresponding
     * request and resolve it's promise.
     *
     * @param response The JSON-RPC response as a string
     */
    recieve(response) {
        if (typeof response === 'string')
            response = JSON.parse(response);
        if (!response.id)
            throw new Error(`Response is missing id: ${response}`);
        const resolve = this.requests[response.id];
        if (!resolve)
            throw new Error(`No request found for response with id: ${response.id}`);
        resolve(response);
        // Clean up by deleting the request entry
        delete this.requests[response.id];
    }
}

/**
 * A `Client` using HTTP/S for communication.
 */
class HttpClient extends Client {
    constructor(server) {
        super();
        this.server = server;
    }
    // Overrides of `Client` methods
    async send(request) {
        return browserPonyfill(this.server, {
            method: 'POST',
            mode: 'cors',
            cache: 'no-cache',
            credentials: 'same-origin',
            headers: {
                'Content-Type': 'application/json; charset=utf-8',
                'Accept': 'application/json; charset=utf-8'
            },
            body: JSON.stringify(request)
        })
            .then(response => response.json())
            .then(response => this.recieve(response));
    }
}

// https://github.com/maxogden/websocket-stream/blob/48dc3ddf943e5ada668c31ccd94e9186f02fafbd/ws-fallback.js

var ws = null;

if (typeof WebSocket !== 'undefined') {
  ws = WebSocket;
} else if (typeof MozWebSocket !== 'undefined') {
  ws = MozWebSocket;
} else if (typeof commonjsGlobal !== 'undefined') {
  ws = commonjsGlobal.WebSocket || commonjsGlobal.MozWebSocket;
} else if (typeof window !== 'undefined') {
  ws = window.WebSocket || window.MozWebSocket;
} else if (typeof self !== 'undefined') {
  ws = self.WebSocket || self.MozWebSocket;
}

var browser = ws;

/**
 * A `Client` using the WebSockets API for communication.
 */
class WebSocketClient extends Client {
    constructor(server) {
        super();
        this.server = server;
        this.socket = new browser(server);
        // @ts-ignore
        this.socket.addEventListener('message', (event) => {
            this.recieve(event.data);
        });
    }
    // Overrides of `Client` methods
    async send(request) {
        if (this.socket.readyState !== browser.OPEN) {
            await new Promise(resolve => {
                this.socket.onopen = () => resolve();
            });
        }
        this.socket.send(JSON.stringify(request));
    }
}

/**
 * A `Client` using the Web Workers API for communication.
 */
class WebWorkerClient extends Client {
    /**
     * Constructor
     *
     * @param server A URL to a Javascript file or a `Worker` instance
     */
    constructor(server) {
        super();
        if (typeof server === 'string')
            server = new Worker(server);
        this.worker = server;
        this.worker.onmessage = (event) => {
            this.recieve(event.data);
        };
    }
    // Overrides of `Client` methods
    send(request) {
        this.worker.postMessage(request);
    }
}

const thing = { type: 'Thing' };
var httpClient = new HttpClient('http://127.0.0.1:2000');
httpClient.execute(thing).then(console.log);
var wsClient = new WebSocketClient('ws://127.0.0.1:2000');
wsClient.execute(thing).then(console.log);
var wwClient = new WebWorkerClient('/webWorkerServer.js');
wwClient.execute(thing).then(console.log);
