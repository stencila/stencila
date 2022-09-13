-- A fixture for testing parsing of create table syntax
-- for deriving parameter properties from columns

CREATE TABLE table_a (
    col_a INTEGER DEFAULT 5 CHECK(col_a >= 1 AND col_a <= 10),
    col_b TEXT DEFAULT 'Hello' NOT NULL,
    col_c REAL NULL DEFAULT 15.6 CHECK(col_c > 10 AND col_c < 20),
    col_d DATE CHECK(col_d > '2000-01-01' AND col_d < '2010-12-31'),
    col_e DATE DEFAULT '2999-01-09' CHECK(col_e < '3000-01-01'::DATE),
    col_f TEXT CHECK(((length(col_f) < 10)))
)
