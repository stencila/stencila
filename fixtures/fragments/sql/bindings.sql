SELECT * FROM table_1 WHERE col_a = $par1 AND col_b = $par_2;

SELECT * FROM table_2
WHERE col_integer = $par_integer
   OR col_text = $par_text


INSERT INTO table_2(col_1, col_2) VALUES ($Par3, $par4);

DELETE FROM table_3 WHERE col_1 = $par_5 AND some_text = "ignored: $1 $123 _$ignore";
