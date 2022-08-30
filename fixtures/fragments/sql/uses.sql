SELECT * FROM table_a;

SELECT * FROM table_b LEFT JOIN table_c ON table_b.col_1 = table_c.col_2;

SELECT * FROM table_d WHERE col_1 > (SELECT col_1 FROM table_e);
