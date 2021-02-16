# RESTful API

## Headers

For all requests, the `Content-Type` of the response is the format specified by the `Accept` header, defaulting to `application/json`.

## `GET /`

Get the content of the stencil.

## `GET /<path>`

Get the content of a file within the stencil folder at path `<path>`. Returns `404` if no file exists at that path.

## `GET /#<id>`

Get the content of the stencil node with `id` property equal to `<id>`. Returns `404` if no nodes has that id. Method: `find`.

## `GET /~<path>`

Get the content of the stencil node at the [JSON Pointer]() path `<path>`. Returns `404` if no node at the path. Method: `select` with `lang: jsonptr`.

## `GET /?<query>`

Get the content of the stencil nodes selected by the [JMESPath]() expression `<query>`. Returns `404` if no nodes selected. Method: `select` with `lang: jmespath`.

## `POST /`

Call a stencil with variables set at the values in the request body.

## `POST /.<method>`

Call a stencil method e.g. `execute` with the arguments in the request body.

## `GET /@`

Get a list of stencil variables.

## `GET /@<name>`

Get the stencil variable with `name`. Returns `404` if no such variable. Method: `get`.

## `POST /@<name>`

Method: `set`.

## `DELETE /@<name>`

Method: `delete`.

## `GET /!`

Get a list of stencil functions. Method: `funcs`.

## `POST /!<name>`

Call the stencil function `name` with arguments in the request body. Method: `call`.
