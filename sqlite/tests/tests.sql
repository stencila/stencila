SELECT load_extension('../stencila.so');

DROP TABLE IF EXISTS test;

CREATE TABLE test (
	a   REAL
);

INSERT INTO test VALUES (1);
INSERT INTO test VALUES (2);
INSERT INTO test VALUES (3);

SELECT stencila_version();
SELECT sqrt(log(a)) FROM test;


