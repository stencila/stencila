---
title: Notes on the `r-sheet` example
abstract: |
  This example illustrates the use of R code within a Stencila Sheet.
---

Stencila allows you seamlessly collaborate on the same document regardless of its format and regardless of the skillset background. In this example we demonstrate how you can enable a spreadsheet user/collaborator to deploy the functions available in R.

The example is based on the introductory course to R from Software and Data Carpentry, and uses standard built-in R functions. However, through Stencila Libraries API it is possible to make practically any R functions available to spreadsheet users.

The `r-sheet.csv` example in plain editor looks like this:

```
,Day 1,Day 2,Day 3,Day 4,Day 5,Day 6,Day 7,Day 8,Day 9,Day 10,Day 11,Day 12,Day 13,Day 14,Day 15,Day 16,Day 17,Day 18,Day 19,Day 20,Day 21,Day 22,Day 23,Day 24,Day 25,Day 26,Day 27,Day 28,Day 29,Day 30,Day 31,Day 32,Day 33,Day 34,Day 35,Day 36,Day 37,Day 38,Day 39,Day 40
Patient 1,0,0,1,3,1,2,4,7,8,3,3,3,10,5,7,4,7,7,12,18,6,13,11,11,7,7,4,6,8,8,4,4,5,7,3,4,2,3,0,0
Patient 2,0,1,2,1,2,1,3,2,2,6,10,11,5,9,4,4,7,16,8,6,18,4,12,5,12,7,11,5,11,3,3,5,4,4,5,5,1,1,0,1
Patient 3,0,1,1,3,3,2,6,2,5,9,5,7,4,5,4,15,5,11,9,10,19,14,12,17,7,12,11,7,4,2,10,5,4,2,2,3,2,2,1,1
Patient 4,0,0,2,0,4,2,2,1,6,7,10,7,9,13,8,8,15,10,10,7,17,4,4,7,6,15,6,4,9,11,3,5,6,3,3,4,2,3,2,1
Patient 5,0,1,1,3,3,1,3,5,2,4,4,7,6,5,3,10,8,10,6,17,9,14,9,7,13,9,12,6,7,7,9,6,3,2,2,4,2,0,1,1
,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
Mean for Patient 1 ,r= mean(B2:AO2),,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
Mean for Patient 2 ,r= mean(B3:AO3),,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
Mean for Patient 3 ,r= mean(B4:AO4),,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
Mean for Patient 4 ,r= mean(B5:AO5),,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
Mean for Patient 5 ,r= mean(B6:AO6),,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
“r= hist(B2:AO2)”,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
“r= hist(B3:AO3)”,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
“r= hist(B4:AO4)”,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
“r= hist(B5:AO5)”,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
“r= hist(B6:AO6)”,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,
```

When previewed in a spreadsheet application, it still allows the user to enter data. The cells starting with `"r=...` include R functions which in a standard spreadsheet application will not work. However, when you open the file in Stencila Sheets, Stencila will call the R execution context and run these functions calculating means and plotting histograms. At the same time, you can still edit cells with data, add more R functions and then save the file in `csv` format (which then you can send back to the collaborator who prefers to use a spreadsheet application).
