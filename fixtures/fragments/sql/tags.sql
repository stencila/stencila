-- To be able to assign the result of query to a document variable
-- @assigns result1
SELECT * FROM table1;

-- To be able to declare reading from a file
-- The COPY statement is not recognised by tree-sitter-sql
-- @reads some/data.csv
COPY table1 FROM 'some/data.csv';

-- To be able to declare writing to a file
-- @writes some/data.csv
COPY table1 TO 'some/data.csv';
