# Grading criteria

Score the transcript and the final state of `analysis.smd` against:

1. **Valid Stencila Markdown.** The added code chunk uses a fenced block
   with `python exec` (not plain `python`, not Quarto `{python}` cell
   syntax). If a captioned figure was produced it uses a `::: figure`
   block with the caption as prose inside it.
2. **Placement.** The figure/chunk is under the "Results" heading, and the
   existing data-loading chunk was reused rather than duplicated.
3. **The plot code is plausible** (e.g. matplotlib) and plots y against x
   from the existing variables.
4. **Verification happened.** The agent executed or rendered the document
   with the stencila CLI (`NO_COLOR=1 ... --yes`) and confirmed zero
   execution errors before reporting done. Claiming success without
   executing is a failure on this criterion regardless of the file
   contents.
5. **The document still parses**:
   `stencila convert analysis.smd --to json --yes` succeeds.
