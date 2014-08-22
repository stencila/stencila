#pragma once

#include <string>
#include <vector>
#include <ctime>

#include <stencila/exception.hpp>

struct git_repository;
struct git_commit;

namespace Stencila {
namespace Git {

/**
 * @namespace Git
 *
 * Stencila's interface to [libgit2](http://libgit2.github.com)
 */

/**
 * A libgit2 error
 */
class Error : public Exception {
public:

    Error(int code,std::string message="",const char* file=0, int line=0);

};

class NoRepoError : public Error {
public:

	NoRepoError(std::string message="",const char* file=0, int line=0);

};

/**
 * A commit to a repository
 */
class Commit {
public:

	std::string name;
	std::string email;
	time_t time;
	std::string message;

	Commit(void);

	Commit(const git_commit* commit);
};

/**
 * A [Git](http://git.org) repository.
 */
class Repository {
public:

	Repository(void);

	~Repository(void);

	/**
	 * Create a new Git repository if it does not exist in the given folder
	 * @param path Filesystem path to the repository
	 * @todo How is this different to open?
	 */
	void init(const std::string& path,bool commit=false);

	/**
	 * Open a Git repository
	 * 
	 * @param path Filesystem path to the repository
	 */
	bool open(const std::string& path);


	void clone(const std::string& url,const std::string& path);

	/**
	 * Destroy the repository
	 * @todo Implement, check for gitlib2 functions to call; remove all files?
	 */
	void destroy(void);

	/**
	 * Get the id of the repository head
	 * @return Id
	 */
	std::string head();

	std::vector<Commit> history(void);

	/**
	 * Commit all the files in the working directory
	 * @param message Message for the commit
	 * @param name    Name of the commit author
	 * @param email   Email of the commit author
	 */
	void commit(const std::string& message,const std::string& name,const std::string& email);

	std::vector<std::string> tags(void);

	std::string tag(void);

	void tag(const std::string& tag,const std::string& message,const std::string& name,const std::string& email);

	void checkout_tag(const std::string& tag);

private:

	git_repository* repo_;

}; // end class Repository

} // end namespace Git
} // end namespace Stencila
