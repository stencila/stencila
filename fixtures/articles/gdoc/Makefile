# This needs to be run with a GOOGLE_TOKEN env var e.g.
#
#   GOOGLE_TOKEN=ya29.A0ARrdaM9iPSUka... make
#
# You can get a token by opening `rust/provider-gdrive/src/picker.html`
# See the instructions in the <body> of that file

all: kitchen-sink-test-fixture.gdoc
     
kitchen-sink-test-fixture.gdoc:
	cargo run --bin providers -- import gdrive:doc/1BePN1If6tUAr1eMg32MJMFKjbuuEkzwBiOxsYp3XqoU $@
