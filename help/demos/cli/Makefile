# Recipes for converting Markdown demo scripts into Asciinema "consolecasts"
#
# Requires pv and asciinema. On Debian:
#    sudo apt-get install pv asciinema

all: $(patsubst %.md,%.cast,$(wildcard *.md))

# Make a Demo Magic script from Markdown source
%.sh: %.md
	csplit --quiet --prefix temp $< /Script/6
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

# Remove temporary files
clean:
	rm -f temp* *.sh *.cast *.svg

# If the .cast files are not created then create them
# Useful for running dev server locallly without having
# to build demos
%.touch:
	touch $*.cast
touch: $(patsubst %.md,%.touch,$(wildcard *.md))
