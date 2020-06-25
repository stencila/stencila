---
id: 4170119
title: eLife Sources
description: Add an eLife article to your Stencila project
published: true
---

![](https://elifesciences.org/assets/patterns/img/patterns/organisms/elife-logo-xs@1x.24c98c55.png)

[eLife](https://elifesciences.org) is a leading publisher that aims to improve research communication through open science and open technology innovation.

You can add an eLife article as a source within your Stencila project. That will enable you to enrich the article, for example, by replacing a static figure with the code that was used to generate it.

# Creating an eLife source

To add an eLife article as a project source:

1. Go to the project **Sources** page

![](http://stencila.github.io/hub/manager/snaps/project-sources-menu-item.png)


2. Press the **New** button and select **From journal** > **eLife**

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-button.png)


3. Enter the number of the eLife article and the path that you want to link it to within the project.

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-elife.png)


# Pulling an eLife source

Once you have added an eLife article source, you can "pull" it to your project. Pulling a project source fetches a copy and stores it in your project's working directory.

For eLife sources, pulling the project will fetch the article's published XML from https://elifesciences.org as well as any images for the figures (in a sibling `.media` folder). For example, after pulling the eLife article number 5000 you would have these files added to your project.

- `elife-article-5000.xml`
- `elife-article-5000.xml.media/elife-05000-fig1.jpg`

# Pushing an eLife source

Stencila does not yet support "pushing" an article to eLife. (i.e. publishing it with them). We are considering ways to enable this feature. If you are interested please get in contact and let us know your ideas!
