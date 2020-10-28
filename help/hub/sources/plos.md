---
id: 4170124
title: PLOS Sources
description: Add a PLOS article to your Stencila project
authors:
  - Colette Doughty <colette@stenci.la> (https://stenci.la)
collectionId: 2414017
published: true
---

![](https://i.imgur.com/1qehB1B.png)

[PLOS](https://plos.org/) is a nonprofit, Open Access publisher empowering researchers to accelerate progress in science and medicine by leading a transformation in research communication.

You can add a PLOS article as a source within your Stencila project. That will enable you to enrich the article, for example, by replacing a static figure with the code that was used to generate it.

## Creating an PLOS source

To add an PLOS article as a project source:

1. Go to the project **Sources** page

    ![](http://stencila.github.io/hub/manager/snaps/project-sources-menu-item.png)

2. Press the **New** button and select **From journal** > **PLOS**

    ![](http://stencila.github.io/hub/manager/snaps/project-sources-new-button.png)

3. Enter the number of the PLOS article (The article's DOI e.g. 10.1371/journal.pcbi.1007676) and the path that you want to link it to within the project (What you want to name your source).

    ![](http://stencila.github.io/hub/manager/snaps/project-sources-new-plos.png)

## Pulling 

By adding a PLOS article source, you would have "pulled" it to your project. Pulling a project source fetches a copy and stores it in your project's working directory.

For PLOS sources, pulling the project will fetch the article's published XML from https://journals.plos.org/ as well as any images for the figures (in a sibling `.media` folder).

## Pushing 

Stencila does not yet support "pushing" an article to PLOS. (i.e. publishing it with them). We are considering ways to enable this feature. If you are interested please get in contact and let us know your ideas!
