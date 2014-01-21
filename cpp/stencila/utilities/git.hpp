#pragma once

#include <string>
#include <vector>
#include <fstream>
#include <ctime>

#include <boost/filesystem.hpp>
#include <boost/algorithm/string/erase.hpp>

#include <git2.h>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Utilities {
namespace Git {

/**
 * @namespace Git
 *
 * Stencila's interface to [libgit2](http://libgit2.github.com)
 */

/**
 * A libgit2 error
 */
class GitError : public Exception {
public:

    GitError(int code,const char* file=0, int line=0):
        Exception("",file,line){
    	const git_error* error = giterr_last();
		const char* message = (error && error->message) ? error->message : "unknown";
		message_ = message;
	}

};
#define STENCILA_GIT_THROW(code) throw GitError(code,__FILE__,__LINE__);
#define STENCILA_GIT_TRY(call) { int code = call; if(code) STENCILA_GIT_THROW(code); }

/**
 * A commit to a repository
 */
class Commit {
public:

	std::string name;
	std::string email;
	time_t time;
	std::string message;

	Commit(void){
	}

	Commit(const git_commit* commit){
		const git_signature* author = git_commit_author(commit);
		name = author->name;
		email = author->email;
		time = git_commit_time(commit);
		message  = git_commit_message(commit);
	}
};

/**
 * A [Git](http://git.org) repository.
 */
class Repository {
private:

	git_repository* repo_;
	bool open_;

public:

	Repository(void):
		repo_(nullptr),
		open_(false){
	}

	~Repository(void){
		if(repo_) git_repository_free(repo_);
	}

	/**
	 * Create a new Git repository if it does not exist in the given folder
	 * @param path Filesystem path to the repository
	 * @todo How is this different to open?
	 */
	void init(const std::string& path,bool commit=false){
		STENCILA_GIT_TRY(git_repository_init(&repo_,path.c_str(),false));
		open_ = true;

		if(commit){
		    git_signature *sig;
		    git_index *index;
		    git_oid tree_id, commit_id;
		    git_tree *tree;

			git_repository_index(&index, repo_);
			git_index_write_tree(&tree_id, index);
			git_tree_lookup(&tree, repo_, &tree_id);
			git_signature_now(&sig,"Anonymous","none");
			git_commit_create_v(&commit_id, repo_, "HEAD", sig, sig, NULL, "Initial commit", tree, 0);

			git_signature_free(sig);
			git_tree_free(tree);
			git_index_free(index);
		}
	}

	/**
	 * Open a Git repository
	 * 
	 * @param path Filesystem path to the repository
	 */
	bool open(const std::string& path){
		if(not open_){
			char path_chars[1024];
			if(git_repository_discover(path_chars,1024,path.c_str(),true,"/")==0){
				STENCILA_GIT_TRY(git_repository_open(&repo_,path_chars));
				open_ = true;
			}
		}
		return open_;
	}

	void open_or_init(const std::string& path,bool commit=false){
		if(not open(path)) init(path,commit);
	}


	void clone(const std::string& url,const std::string& path){
		STENCILA_GIT_TRY(git_clone(&repo_, url.c_str(), path.c_str(), NULL));
	}

	/**
	 * Destroy the repository
	 * @todo Implement, check for gitlib2 functions to call; remove all files?
	 */
	void destroy(void){
		repo_ = 0;
	}

	/**
	 * Get the id of the repository head
	 * @return Id
	 */
	std::string head(){
		git_reference* ref;
		git_reference_lookup(&ref, repo_, "refs/heads/master");
		if(!ref) return "<none>";
		const git_oid* oid = git_reference_target(ref);
		git_reference_free(ref);
		char buffer[256];
		git_oid_tostr(buffer,256,oid);
		return buffer;
	}

	std::vector<Commit> log(void){
		// Get the oid of the HEAD
		git_oid oid;
		git_oid_fromstr(&oid, head().c_str());
		// Set up to walk the tree in topological order
		git_revwalk* walker;
		git_revwalk_new(&walker, repo_);
		git_revwalk_sorting(walker, GIT_SORT_TOPOLOGICAL);
		git_revwalk_push(walker, &oid);
		// Do the walk
		std::vector<Commit> log;
		git_commit* commit;
		while ((git_revwalk_next(&oid, walker)) == 0) {
			int error = git_commit_lookup(&commit, repo_, &oid);
			log.push_back(Commit(commit));
			git_commit_free(commit);
		}
		git_revwalk_free(walker);
		return log;
	}


	/**
	 * Create a file within the repository's working directory
	 * @param path Filesystem path within the working directory
	 */
	void make(const std::string& path){
		std::string root = git_repository_workdir(repo_);
		boost::filesystem::path path_full(root);
		path_full /= path;
		if(!exists(path_full)){
			std::ofstream file(path_full.string());
			file<<"\n";
			file.close();
		}
	}

	/**
	 * Create several files within the repository's working directory
	 * @param path Filesystem path within the working directory
	 * @param others More paths
	 */
	template<typename... Others>
	void make(const std::string& path, Others... others){
		make(path);
		make(others...);
	}

	/**
	 * Commit all the files in the working directory
	 * @param message Message for the commit
	 * @param name    Name of the commit author
	 * @param email   Email of the commit author
	 */
	void commit(const std::string& message,const std::string& name,const std::string& email){
		// Get index file for repository 
		git_index* index;
		STENCILA_GIT_TRY(git_repository_index(&index, repo_));
		// Add or update an index entry for each file in the working directory
		std::string root = git_repository_workdir(repo_);
		for(boost::filesystem::recursive_directory_iterator end, dir(root); dir != end; ++dir) {
		    // Convert path to a relative path
		    std::string path = boost::filesystem::path(*dir).string();
		    boost::erase_first(path,root);
		    // Don't include .git directory
		    if(path.substr(0,4)==".git") continue;
		    if(is_regular_file(dir->status())){
		     	git_index_add_bypath(index, path.c_str());
		    }
		}
		// Build a tree from the index
		git_oid tree_oid;
		git_tree* tree;
		STENCILA_GIT_TRY(git_index_write_tree(&tree_oid, index));
		STENCILA_GIT_TRY(git_tree_lookup(&tree, repo_, &tree_oid));
		// Create an author signatue
		git_signature* author;
		STENCILA_GIT_TRY(git_signature_now(&author,
			name.length()==0?"Anonymous":name.c_str(), //name of the person; fails if zero length
			email.length()==0?"none":email.c_str() //email of the person; fails if zero length
		));
		// Determine parent commit (if any)
		git_reference* ref;
		git_reference_lookup(&ref, repo_, "refs/heads/master");
		if(!ref){
			// No parent, so just create the commit
			git_oid commit_oid;
			STENCILA_GIT_TRY(git_commit_create_v(
				&commit_oid,
				repo_,
				"refs/heads/master",
				author,
				author,
				"UTF-8",
				message.c_str(),
				tree,
				0
			));
		} else {
			// Get the parent
			const git_oid* parent_oid = git_reference_target(ref);
			git_commit* parent;
			git_commit_lookup(&parent,repo_,parent_oid);
			// Create the commit
			git_oid commit_oid;
			STENCILA_GIT_TRY(git_commit_create_v(&commit_oid,repo_,
				"refs/heads/master",
				author,
				author,
				"UTF-8",
				message.c_str(),
				tree,
				1,parent
			));
		}
		// Free memory
		git_signature_free(author);
		git_tree_free(tree);
		git_index_free(index);
		git_reference_free(ref);
	}

	std::string tag(void){
		git_strarray tags;
		STENCILA_GIT_TRY(git_tag_list(&tags, repo_));
		std::string tag = "";
		if(tags.count>0) tag = tags.strings[tags.count-1];
		git_strarray_free(&tags);
		return tag;
	}

	void tag(const std::string& tag,const std::string& message,const std::string& name,const std::string& email){
		
		git_object* target = nullptr;		
		STENCILA_GIT_TRY(git_revparse_single(&target, repo_, "HEAD^{commit}"));
		
		git_signature* tagger = nullptr;
		STENCILA_GIT_TRY(git_signature_now(&tagger,
			name.length()==0?"Anonymous":name.c_str(), //name of the person; fails if zero length
			email.length()==0?"none":email.c_str() //email of the person; fails if zero length
		));

		git_oid tag_oid;
		STENCILA_GIT_TRY(git_tag_create(
			&tag_oid,
			repo_,
			tag.length()==0?"tag":tag.c_str(),
			target,
			tagger,
			message.length()==0?"Tagged":message.c_str(),
			false
		));

		git_object_free(target);
		git_signature_free(tagger);
	}

}; // end class Repository

} // end namespace Git
} // end namespace Utils
} // end namespace Stencila
