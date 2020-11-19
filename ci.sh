#!/bin/sh

# Bash functions for doing things as part of CI build

checkout_pages() {
  git config --global user.email "ci@stenci.la"
  git config --global user.name "Stencila CI Bot"

  git remote add origin-pages https://${GITHUB_TOKEN}@github.com/stencila/schema.git > /dev/null 2>&1
  git fetch --all
  git checkout --force --track origin-pages/gh-pages
  git branch --set-upstream-to origin-pages/gh-pages
  git reset --hard
}

push_pages() {
  git push --quiet
}

commit_docs() {
  git add -f --all docs
  git commit --message "$1"
}

deploy_typescript_docs_to_pages() {
  checkout_pages

  cp public/packages-index.html index.html
  git add index.html
  git commit --message "docs(Package index): Update"

  cd ts
  commit_docs "docs(Typescript): Update"

  push_pages
}

deploy_python_docs_to_pages() {
  checkout_pages
  commit_docs "docs(Python): Update"
  push_pages
}

deploy_r_docs_to_pages() {
  checkout_pages
  commit_docs "docs(R): Update"
  push_pages
}

# Call one of the functions as specified by the first arg
$1
