A parameter can be represented in Markdown using a pair of forward slashes and optionally, curly braces defining options.

Boolean parameters /par1/{bool def=true} and /par2/{bool def=false}.

Integer parameters /parA/{int min=1 max=100 mult=2 def=2} and /parB/{int max=100 def=42}.

Number parameters /par3/{num min=1 max=100 mult=2 def=22} and /par4/{num min=-1 max=4 def=3.14}.

String parameters /par5/{str min=1 max=100} and /par6/{str min=1 max=20 pattern="[A-Za-z ]+"}.

For enum parameters set the `values` property using a JSON5 array e.g. /par7/{enum vals=["One option","Another option"]} (allows for spaces in values) or using a comma separated string e.g. /par8/{enum vals=["A","B","C","D"]}. Note that when using JSON/5 for values they can be any type, not just strings e.g. /par9/{enum vals=[1,"two",3.14]}.

When there is no `type` option, parameters will have no validator /par10/{}. The curly braces are required to avoid forward slash file paths e.g. /some/path, /another/path/ being parsed as parameters.
