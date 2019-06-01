# Run a demo interactively e.g. for a tutorial
# Requires that you press enter when you want the demo
# to progress. This turns off simulated typing.
run-%: demo-magic.sh
	./$*.sh -d

# Create a Markdown document from the demo script
# Converts headers, comments and commands to Markdown
# and ignores everything else. Also unescapes backticks.
md-%:
	@sed -r \
	    -e 's/\\`/`/g' \
		-e 's/^h1 "(.*)"/# \1/g' \
		-e 's/^h2 "(.*)"/## \1/g' \
		-e 's/^h3 "(.*)"/### \1/g' \
		-e 's/^c "(.*)"/\1/g' \
	    -e 's/^e "(.*)"/\n```bash\n\1\n```/g' \
		-e '/^(#!|\.|z|p).*/d' $*.sh > $*.md

# Preview a demo before recording it
preview-%: demo-magic.sh
	./$*.sh -n

# Records an asciinema cast
record-%: asciinema demo-magic.sh
	asciinema rec -c "./$*.sh -n" --overwrite $*.cast

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

# Download the demo-magic.sh script
demo-magic.sh:
	curl https://raw.githubusercontent.com/paxtonhare/demo-magic/master/demo-magic.sh > demo-magic.sh
