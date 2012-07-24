#include <sqlite3.h>
#include <sqlite3ext.h>

SQLITE_EXTENSION_INIT1

#include "extension-functions.c"

const char* STENCILA_SQLITE_VERSION = "0.0.0";

/*
@return The version number of this library
*/
static void stencila_sqlite_version(
  sqlite3_context *context,
  int argc,
  sqlite3_value **argv
){
  sqlite3_result_text(context,STENCILA_SQLITE_VERSION,-1,0);
}

int sqlite3_extension_init(
  sqlite3 *db,
  char **pzErrMsg,
  const sqlite3_api_routines *pApi
){
  SQLITE_EXTENSION_INIT2(pApi)

  //Register functions
  sqlite3_create_function(db, "stencila_sqlite_version", 0, SQLITE_ANY, 0, stencila_sqlite_version, 0, 0);
  RegisterExtensionFunctions(db);

  return 0;
}

