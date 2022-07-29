A parameter can be represented in Markdown using a pair of forward slashes and optionally, curly braces defining options.

Boolean parameters /par1/{type=boolean default=true} and /par2/{bool default=false}.

Number parameters /par3/{type=number min=1 max=100 step=2} and /par4/{num default=0 min=-1 max=4}.

String parameters /par5/{type=string min-length=1 maxLength=100} and /par6/{str value='Hello world' pattern="[a-z]+"}.

For enum parameters set the `values` property using a JSON5 array e.g. /par7/{type=enum values=['One option', 'Another option']} (allows for spaces in values) or using a comma separated string e.g. /par8/{enum values="A,B,C,D"}. Note that when using JSON/5 for values they can be any type, not just strings e.g. /par9/{enum values=[1, 'two', 3.14]}.

When there is no `type` option, parameters will have no validator /par10/{}. The curly braces are required to avoid forward slash file paths e.g. /some/path, /another/path/ being parsed as parameters.
