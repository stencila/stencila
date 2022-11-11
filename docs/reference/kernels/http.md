The HTTP kernel...

# Use cases

Mostly intended as a base for other kernels that use rest API, notably the PostgREST kernel.

# Request line

# Headers

# Comments

As an extension to HTTP/1.1, the HTTP kernel treats lines which start with the `#` character, and are before the body, as comments. You may want to use comments to describe the purpose of the request e.g.

```http exec
# Get a randomly generated UUID from httpbin.org
GET uuid
Host: https://httpbin.org
```

# Tags

Another reason to use comment lines is to use tags. In particular using `@global` tags save you from having to add headers for every request.

The following code chunk defines the `@host` tag as global,

```http exec
# @global @host https://httpbin.org
```

This means that all HTTP chunks or expressions in this document that will fallback to using https://httpbin.org if the URL does not have a host in it, or there is no `Host` header e.g.

```http exec
GET uuid
```

The `@assign` tag can be used. Note that because we have declared that this code chunk assigns a variable, it does not have any outputs.

```http exec
# @assigns myid
GET uuid
```

We can use the assigned variable in other languages. For example, here we output the UUID in uppercase letters,

```python exec
myid['uuid'].upper()
```

# Variable interpolation

The HTTP kernel will also do

&[status]{enum vals=[200,401,402,403,500]}

```http exec
GET status/$status
```

In the body of the request, all variables will be interpolated as JSON. This means that you don't have to put double quotes around interpolated strings in JSON payloads. For example, here is a string parameter for a greeting:

&[greeting]{ label="Greeting"}

And this code chunk puts that greeting into a JSON payload that will get echoed back in the response

```http exec
POST post
Content-Type: application/json

{"greeting" : $greeting }
```
