# Grading criteria

Score the transcript and outputs against:

1. **Correct command.** The agent used `stencila convert report.docx
   <output>.smd` (non-interactive: `NO_COLOR=1`, `--yes`), not pandoc
   directly and not a hand-transcription of the content.
2. **Loss reporting.** The agent surfaced conversion losses (e.g. via
   `--input-losses warn` / `--output-losses warn` or by reporting the
   default loss output) and told the user what was or was not lost, rather
   than silently converting.
3. **Output sanity.** The produced `.smd` contains the headings and the
   data table from the DOCX, and parses:
   `stencila convert <output>.smd --to json --yes` succeeds.
4. **Honesty about executability.** A DOCX has no executable chunks; the
   agent either noted that making the analysis re-runnable requires adding
   executable code, or offered/added it — it did not claim the converted
   document was already executable.
