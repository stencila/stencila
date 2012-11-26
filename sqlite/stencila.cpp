#include <sqlite3.h>
#include <sqlite3ext.h>
SQLITE_EXTENSION_INIT1

#include <stencila/version.hpp>
#include <stencila/dataset-math-functions.hpp>
#include <stencila/dataset-math-aggregators.hpp>

/*
The version number of the Stencila library
*/
static void stencila_version(sqlite3_context *context, int argc, sqlite3_value **argv){
  sqlite3_result_text(context,Stencila::version,-1,0);
}

/*
Extension module initialisation function
*/
extern "C"
int sqlite3_extension_init(sqlite3 *db, char **pzErrMsg, const sqlite3_api_routines *pApi){
    SQLITE_EXTENSION_INIT2(pApi)

    sqlite3_create_function(db, "stencila_version", 0, SQLITE_ANY, 0, stencila_version, 0, 0);
    Stencila::MathFunctions::create(db);
    Stencila::MathAggregators::create(db);

    return 0;
}
