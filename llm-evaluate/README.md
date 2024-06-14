# LLM Evaluate

A pipeline for generating routing information for LLMs.

## Overview

The pipeline has three stages:

1. **Gather**. First, we gather information evaluates about how well various
   LLMs perform in terms of cost, speed, and quality, across a variety of
   contexts (coding or text, for example). Currently, we scrape information from
   various places. In the future, we may generate our own statistics.
2. **Measure**. We do a grid-search, using weightings for cost, speed, and
   quality to establish which LLM best meets the trade-offs for a given
   context.
3. **Export**. We export the results in JSON, to be used as routing information
   for Stencila's router. The router can then direct any request for assistance
   to the LLM that best meets the user's requested constraints.

Intermediate information is stored in a SQL database, ensuring we can track,
and reproduce, the results.

## Usage

Very simple notes for now:

- We use PDM to manage the Python environment (use Make as with other python
  packages in Stencila)
- You will get access to a command line app call [`lemmy`](https://en.wikipedia.org/wiki/Lemmy)
- You can run `lemmy --help` to see the available commands
- You will need to set "LEMMY_DATABASE_PATH" in your environment.
- Run `lemmy scrape trustbits` to scrape the trustbits data
- Run `lemmy show provider` to see the scrape records.
- You can run `lemmy generate {id}` to generate the routing records for the LLMs
- Run `lemmy export {id} {json_path}` to export the routing records to a JSON file

For Now: A example `routing.json` is in the root directory.

## Methodology

Here is how we currently calculate the routing information.

- Gather cost, speed, and quality information for each LLM.
- Normalize the cost, speed, and quality information (this allows us to ignore
  the particulars of the units).
- Generate a grid of possible weightings for cost, speed, and quality (these
  must add to 1).
- Calculate a scare as the weighted sum of the normalized cost, speed, and
  quality for each LLM for each set of weightings.
- Select the LLM with the highest score.

