# LLM Evaluate

A pipeline for generating routing information for LLMs.

## Install and Run

Very simple notes for now:

- We use PDM to manage the Python environment (use Make as with other python packages in Stencila)
- You will get access to a command line app call `lemmy`
- You can run `lemmy --help` to see the available commands
- You will need to set "LEMMY_DATABASE_PATH" in your environment.
- Run `lemmy scrape trustbits` to scrape the trustbits data
- Run `lemmy show provider` to see the scrape records.
- You can run `lemmy generate {id}` to generate the routing records for the LLMs
- Run `lemmy export {id} {json_path}` to export the routing records to a JSON file

For Now: A example `routing.json` is in the root directory.

## Methodology

TBD