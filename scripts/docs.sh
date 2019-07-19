#!/bin/bash

# Generate HTML documentation from Markdown files

# Process schema/*.md files to built/*.html files
for filename in schema/*.md; do
  encoda process $filename built/$(basename -s .md $filename).html
done

# Copy over outputs e.g. from export directives
cp schema/*.out.* built/
