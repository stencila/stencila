# Make testing database-ORM bindings using a temporary sqlite
# database
testing-db:
	rm -f src/testing_db/temp.db
	cat src/testing_db/schema.sql | sqlite3 src/testing_db/temp.db
	sea-orm-cli generate entity \
				--database-url sqlite://src/testing_db/temp.db \
				--output-dir src/testing_db
	rm src/testing_db/temp.db
