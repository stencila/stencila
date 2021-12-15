A parameter can be represented in Markdown using a pair of forward slashes and optionally, curly braces defining options.

Boolean parameters /par1/{type=boolean default=true} and /par2/{bool default=false}.

Number parameters /par3/{type=number min=1 max=100 step=2} and /par4/{num default=0 min=-1 max=4}.

String parameters /par5/{type=string min-length=1 maxLength=100} and /par6/{str value='Hello world' pattern="[a-z]+"}.

When there is no `type` option, parameters will have no validator /par7/.
