#!/bin/sh

checkout_pages() {
  git config --global user.email $GIT_AUTHOR_EMAIL
  git config --global user.name $GIT_AUTHOR_NAME
  git checkout gh-pages
}

commit_and_push_pages() {
  git commit --message $1
  git remote add origin-pages https://${GITHUB_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git > /dev/null 2>&1
  git push --quiet --set-upstream origin-pages gh-pages
}
