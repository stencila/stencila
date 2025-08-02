# Makefile for downloading GitHub API examples

NUMBERS := 1 2 3

SEARCH_CODE_FILES := $(foreach n,$(NUMBERS),search-code-$(n).json)
SEARCH_USERS_FILES := $(foreach n,$(NUMBERS),search-users-$(n).json)
SEARCH_REPOSITORIES_FILES := $(foreach n,$(NUMBERS),search-repositories-$(n).json)

ALL_FILES := $(SEARCH_CODE_FILES) $(SEARCH_USERS_FILES) $(SEARCH_REPOSITORIES_FILES)

GITHUB_API := https://api.github.com

define CURL_CMD
curl -s \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  $(if $(GITHUB_TOKEN),-H "Authorization: Bearer $(GITHUB_TOKEN)") \
  $(1) | jq '.'
endef

.PHONY: all search-code search-users search-repositories

# Download all example types
all: $(ALL_FILES)

# Download specific entity types
search-code: $(SEARCH_CODE_FILES)
search-users: $(SEARCH_USERS_FILES)
search-repositories: $(SEARCH_REPOSITORIES_FILES)

# Pattern rules for downloading search code responses
search-code-1.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/code?q=pandas+extension:ipynb&per_page=1") > $@

search-code-2.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/code?q=def+main+language:python&per_page=1") > $@

search-code-3.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/code?q=data.frame+language:r&per_page=1") > $@

# Pattern rules for downloading search users responses
search-users-1.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/users?q=tom+repos:>42+followers:>1000&per_page=1") > $@

search-users-2.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/users?q=type:org+followers:>100000&per_page=1") > $@

search-users-3.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/users?q=location:seattle+language:python&per_page=1") > $@

# Pattern rules for downloading search repositories responses
search-repositories-1.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/repositories?q=tetris+language:assembly&per_page=1") > $@

search-repositories-2.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/repositories?q=topic:api+stars:>10000&per_page=1") > $@

search-repositories-3.json:
	$(call CURL_CMD,"$(GITHUB_API)/search/repositories?q=org:github+archived:false&per_page=1") > $@

# Clean up downloaded files
clean:
	rm -f *.json
