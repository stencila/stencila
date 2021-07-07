all: elife-small.json era-plotly.json

elife-small.json:
	encoda convert https://elifesciences.org/articles/60912 $@

era-plotly.json:
	encoda convert https://hub.stenci.la/api/projects/1316/snapshots/14/files/era.ipynb $@

clean:
	rm -rf *.json*
