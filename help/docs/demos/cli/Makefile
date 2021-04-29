# Recipes for converting Mrkdown demo scripts into Asciinema "consolecasts"
#
# Requires pv and asciinema. On Debian:
#    sudo apt-get install pv asciinema
# To generate GIFs requires `docker`.

all: $(patsubst %.md,%.gif,$(wildcard *.md))

# Install node deps
node_modules: package.json
	npm install

# Make a Demo Magic script from Markdown source
%.sh: %.md node_modules
	csplit -f temp $< /Script/6
	npx encoda convert --from md temp01 --to demo-magic $@
	chmod +x $@
.PRECIOUS: %.sh

# Run a demo interactively e.g. for a tutorial
# Requires that you press enter when you want the demo
# to progress. This turns off simulated typing.
%.run: %.sh
	./$*.sh -d

# Preview a demo before recording it
%.preview: %.sh
	./$*.sh -n

# Record an asciinema cast
%.cast: %.sh
	asciinema rec -c "./$*.sh -n" --overwrite $*.cast
.PRECIOUS: %.cast

# Play an asciinema cast
%.play: %.cast
	asciinema play $*.cast

# Convert a cast to an animated SVG
%.svg: %.cast node_modules
	cat $*.cast | npx svg-term --window --height 25 --width 90 --out $*.svg

# Convert a cast to an animated GIF
%.gif: %.cast
	docker run --rm -v $$PWD:/data asciinema/asciicast2gif $*.cast $*.gif

# Upload an asciinema cast
%.upload: %.cast
	asciinema upload $*.cast

# Remove temporary files
clean:
	rm -f temp* *.sh *.cast *.svg
