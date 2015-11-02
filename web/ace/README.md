This directory contains syntax and theme definitions for Cila for use with [ACE editor](http://ace.c9.io/).

Particularly for development, it is easiest to have these definition files live within an ACE repository. Originally, we had a them in a fork of ACE, but rather than maintain a separate repo, have brought them in here. Now, when building the Stencila `web` module, these files are symlinked into the ACE directory (in `web/other_modules/ace`) by the `Makefiles` `web-ace-patch` recipe.

After doing a `make web-ace-patch`, the syntax definition can be interactively developed using:

	cd web/other_modules/ace
	./static.py --puttable='*'
	open http://127.0.0.1:8888/tool/mode_creator.html
