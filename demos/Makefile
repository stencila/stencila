# Make a Demo Magic script from Markdown source
%.sh: %.md
	npx encoda convert ./$< ./$@ --to demo-magic
	chmod +x $@

# Run a demo interactively e.g. for a tutorial
# Requires that you press enter when you want the demo
# to progress. This turns off simulated typing.
run-%: %.sh
	PATH=$$PATH:$$(pwd) ./$*.sh -d

# Preview a demo before recording it
preview-%: %.sh
	PATH=$$PATH:$$(pwd) ./$*.sh -n

# Records an asciinema cast
record-%: %.sh asciinema
	asciinema rec -c "PATH=$$PATH:$$(pwd) ./$*.sh -n" --overwrite $*.cast

# Play an asciinema cast
play-%: asciinema
	asciinema play $*.cast

# Upload an asciinema cast
upload-%: asciinema
	asciinema upload $*.cast

# Check if asciinema is installed
asciinema:
	@if ! [ -x "$$(command -v asciinema)" ]; then \
		echo "Error: asciinemas is not installed. See https://asciinema.org/docs/installation"; \
	fi
