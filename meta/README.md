# `meta`

A module which specifies the attributes of component classes (e.g `Stencil`, `Sheet`) so that consistent C++ wrappers and documentation can be generated across target environments (e.g. Python, R, Node.js, Web API) whilst staying [DRY](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself).

Each component class is defined in a YAML file. The structure for that file is our own and currently evolving as we implement wrappers and learn what works best.

Each module (e.g. `node`) implements its own wrapper generator (e.g. `node/wrap.js`) which consumes the YAML files and produces wrapping code and documentation. These will usually be written in the module's own language (e.g. Javascript) and use templating engines for code and documentation generation.
