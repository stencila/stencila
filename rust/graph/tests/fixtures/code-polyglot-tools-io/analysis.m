import stats.*

tbl = readtable("data/raw/samples.tsv");
count = height(tbl);
writetable(table(count), "results/matlab-summary.tsv");
