use rquickjs::{Ctx, function::Func};

/// JavaScript source that wires up Rust-backed polyfill functions into
/// proper classes (URL, URLSearchParams, TextEncoder, TextDecoder).
///
/// The Rust functions are injected as `__codemode_polyfill_*` globals,
/// then this JS wraps them in ergonomic class APIs and removes the raw helpers.
const POLYFILLS_JS: &str = r#"
(function() {
    const _parseUrl = globalThis.__codemode_parse_url;
    const _encodeUtf8 = globalThis.__codemode_encode_utf8;
    const _decodeUtf8 = globalThis.__codemode_decode_utf8;
    delete globalThis.__codemode_parse_url;
    delete globalThis.__codemode_encode_utf8;
    delete globalThis.__codemode_decode_utf8;

    // ---- URLSearchParams ----
    class URLSearchParams {
        constructor(init) {
            this._params = [];
            if (typeof init === 'string') {
                init = init.replace(/^\?/, '');
                if (init) {
                    for (const pair of init.split('&')) {
                        const eq = pair.indexOf('=');
                        if (eq === -1) {
                            this._params.push([decodeURIComponent(pair), '']);
                        } else {
                            this._params.push([
                                decodeURIComponent(pair.slice(0, eq)),
                                decodeURIComponent(pair.slice(eq + 1))
                            ]);
                        }
                    }
                }
            } else if (init instanceof URLSearchParams) {
                this._params = [...init._params.map(p => [...p])];
            } else if (Array.isArray(init)) {
                this._params = init.map(([k, v]) => [String(k), String(v)]);
            } else if (init && typeof init === 'object') {
                for (const [k, v] of Object.entries(init)) {
                    this._params.push([String(k), String(v)]);
                }
            }
        }
        append(name, value) { this._params.push([String(name), String(value)]); }
        delete(name) { this._params = this._params.filter(([k]) => k !== name); }
        get(name) { const p = this._params.find(([k]) => k === name); return p ? p[1] : null; }
        getAll(name) { return this._params.filter(([k]) => k === name).map(([, v]) => v); }
        has(name) { return this._params.some(([k]) => k === name); }
        set(name, value) {
            let found = false;
            this._params = this._params.reduce((acc, [k, v]) => {
                if (k === name) {
                    if (!found) { acc.push([k, String(value)]); found = true; }
                } else { acc.push([k, v]); }
                return acc;
            }, []);
            if (!found) this._params.push([String(name), String(value)]);
        }
        sort() { this._params.sort((a, b) => a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0); }
        toString() {
            return this._params
                .map(([k, v]) => encodeURIComponent(k) + '=' + encodeURIComponent(v))
                .join('&');
        }
        entries() { return this._params[Symbol.iterator](); }
        keys() { return this._params.map(([k]) => k)[Symbol.iterator](); }
        values() { return this._params.map(([, v]) => v)[Symbol.iterator](); }
        [Symbol.iterator]() { return this.entries(); }
        forEach(callback, thisArg) {
            for (const [key, value] of this._params) {
                callback.call(thisArg, value, key, this);
            }
        }
        get size() { return this._params.length; }
    }
    globalThis.URLSearchParams = URLSearchParams;

    // ---- URL (backed by Rust url crate) ----
    class URL {
        constructor(url, base) {
            // _parseUrl returns a JSON string or throws
            const parsed = JSON.parse(_parseUrl(url, base || ''));
            this._protocol = parsed.protocol;
            this._hostname = parsed.hostname;
            this._port = parsed.port;
            this._pathname = parsed.pathname;
            this._hash = parsed.hash;
            this._username = parsed.username;
            this._password = parsed.password;
            this._search = parsed.search;
            this.searchParams = new URLSearchParams(this._search);
        }
        get protocol() { return this._protocol; }
        get hostname() { return this._hostname; }
        get port() { return this._port; }
        get pathname() { return this._pathname; }
        get hash() { return this._hash; }
        get username() { return this._username; }
        get password() { return this._password; }
        get host() { return this._port ? this._hostname + ':' + this._port : this._hostname; }
        get origin() { return this._protocol + '//' + this.host; }
        get search() {
            const s = this.searchParams.toString();
            return s ? '?' + s : '';
        }
        get href() {
            let url = this._protocol + '//';
            if (this._username) {
                url += this._username;
                if (this._password) url += ':' + this._password;
                url += '@';
            }
            url += this.host + this._pathname;
            const search = this.searchParams.toString();
            if (search) url += '?' + search;
            if (this._hash) url += this._hash;
            return url;
        }
        toString() { return this.href; }
        toJSON() { return this.href; }
    }
    globalThis.URL = URL;

    // ---- TextEncoder (backed by Rust) ----
    class TextEncoder {
        get encoding() { return 'utf-8'; }
        encode(str) {
            if (str === undefined || str === null) str = '';
            const json = _encodeUtf8(String(str));
            return new Uint8Array(JSON.parse(json));
        }
    }
    globalThis.TextEncoder = TextEncoder;

    // ---- TextDecoder (backed by Rust) ----
    class TextDecoder {
        #encoding;
        constructor(encoding) {
            encoding = (encoding || 'utf-8').toLowerCase();
            if (encoding !== 'utf-8' && encoding !== 'utf8') {
                throw new RangeError("TextDecoder: unsupported encoding '" + encoding + "'");
            }
            this.#encoding = encoding;
        }
        get encoding() { return 'utf-8'; }
        decode(input) {
            if (!input) return '';
            // Convert typed array to plain array of bytes for the Rust function
            const bytes = Array.from(new Uint8Array(input.buffer || input));
            return _decodeUtf8(JSON.stringify(bytes));
        }
    }
    globalThis.TextDecoder = TextDecoder;
})();
"#;

/// Inject Rust-backed polyfill helper functions and evaluate the JS wrappers.
///
/// Uses the `url` crate for URL parsing and Rust's native UTF-8
/// string handling for TextEncoder/TextDecoder.
pub(super) fn inject_polyfills(ctx: &Ctx<'_>) -> Result<(), rquickjs::Error> {
    // URL parser using the `url` crate
    let parse_url = Func::from(|url_str: String, base_str: String| {
        let parsed = if base_str.is_empty() {
            url::Url::parse(&url_str)
        } else {
            url::Url::parse(&base_str).and_then(|base| base.join(&url_str))
        };

        match parsed {
            Ok(u) => {
                let result = serde_json::json!({
                    "protocol": u.scheme().to_string() + ":",
                    "hostname": u.host_str().unwrap_or(""),
                    "port": u.port().map(|p| p.to_string()).unwrap_or_default(),
                    "pathname": u.path(),
                    "hash": if u.fragment().is_some() {
                        format!("#{}", u.fragment().unwrap_or(""))
                    } else {
                        String::new()
                    },
                    "username": u.username(),
                    "password": u.password().unwrap_or(""),
                    "search": u.query().unwrap_or(""),
                });
                Ok::<String, rquickjs::Error>(result.to_string())
            }
            Err(e) => Err(rquickjs::Error::new_from_js_message(
                "value",
                "URL",
                &format!("Invalid URL: {e}"),
            )),
        }
    });

    // TextEncoder: string → UTF-8 bytes as JSON array
    let encode_utf8 = Func::from(|s: String| -> String {
        let bytes: Vec<u8> = s.into_bytes();
        serde_json::to_string(&bytes).unwrap_or_else(|_| "[]".into())
    });

    // TextDecoder: JSON array of bytes → string
    let decode_utf8 = Func::from(|json_bytes: String| -> Result<String, rquickjs::Error> {
        let bytes: Vec<u8> = serde_json::from_str(&json_bytes)
            .map_err(|e| rquickjs::Error::new_from_js_message("value", "string", &e.to_string()))?;
        String::from_utf8(bytes)
            .map_err(|e| rquickjs::Error::new_from_js_message("value", "string", &e.to_string()))
    });

    ctx.globals().set("__codemode_parse_url", parse_url)?;
    ctx.globals().set("__codemode_encode_utf8", encode_utf8)?;
    ctx.globals().set("__codemode_decode_utf8", decode_utf8)?;

    // Evaluate the JS wrappers
    ctx.eval::<(), _>(POLYFILLS_JS)?;

    Ok(())
}
