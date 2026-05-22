process qc {
  input:
  path "data/reads.fastq"
  output:
  path "results/qc/M1-qc.txt"
  script:
  """
  wc -l data/reads.fastq > results/qc/M1-qc.txt
  """
}

process classify {
  input:
  path "results/qc/M1-qc.txt"
  output:
  path "results/classify/M1-taxa.tsv"
  shell:
  """
  printf 'taxon\tcount\nBacteria\t1\n' > results/classify/M1-taxa.tsv
  """
}

process summarize {
  input:
  path "results/classify/M1-taxa.tsv"
  output:
  path "results/summary/report.html"
  exec:
  println "summarize"
}
