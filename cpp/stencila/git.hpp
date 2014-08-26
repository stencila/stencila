#pragma once

#include <string>
#include <vector>
#include <ctime>

#include <stencila/exception.hpp>

// Declarations to avoid including git2.h
struct git_repository;
struct git_commit;

namespace Stencila {
namespace Git {

/**
 * @namespace Stencila::Git
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

/**
 * A libgit2 no repository found error
 */
class NoRepoError : public Error {
public:

	NoRepoError(std::string message="",const char* file=0, int line=0);

};

/**
 * A commit to a Git repository
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
	 * Create a new repository if it does not exist in the directory
	 * 
	 * @param path Filesystem path to the repository
	 */
	void init(const std::string& path,bool commit=false);

	/**
	 * Open a repository
	 * 
	 * @param path Filesystem path to the repository
	 */
	bool open(const std::string& path);

	/**
	 * Open a repository
	 * 
	 * @param url URL of remote repository
	 * @param path Filesystem path to the repository
	 */
	void clone(const std::string& url,const std::string& path);

	/**
	 * Destroy the repository
	 */
	void destroy(void);

	/**
	 * Get the id of the repository head
	 */
	std::string head();

	/**
	 * Get a commit history for the repository
	 */
	std::vector<Commit> history(void);

	/**
	 * Commit all the files in the working directory
	 * 
	 * @param message Message for the commit
	 * @param name    Name of the commit author
	 * @param email   Email of the commit author
	 */
	void commit(const std::string& message,const std::string& name,const std::string& email);

	/**
	 * Get a list of all tags on the repository
	 */
	std::vector<std::string> tags(void);

	/**
	 * Get the latest tag on the repository
	 */
	std::string tag(void);

	/**
	 * Set a tag on the HEAD
	 * 
	 * @param tag     Tag to set
	 * @param message Message to associate with tag
	 * @param name    Name of tagger
	 * @param email   Email of tagger
	 */
	void tag(const std::string& tag,const std::string& message,const std::string& name,const std::string& email);

	/**
	 * Checkout a specific tag 
	 * 
	 * @param tag Tag to checkout
	 */
	void checkout(const std::string& tag);

private:

	git_repository* repo_;

}; // end class Repository

} // end namespace Git
} // end namespace Stencila
