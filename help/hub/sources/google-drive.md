---
title: Google Drive Source
description: Add a Google Drive to your Stencila project
id: 4170127
relatedArticles:
  - ../projects/manage-project-sources.md
authors:
  - 'Colette Doughty <colette@stenci.la> (https://stenci.la)'
---

You can add a Google Drive file or folder as a source within your Stencila project.

Before you start make sure your google account is connected to your Stencila account. To check, go to your account **Settings** in the top right corner of your screen, scroll to the bottom of the page and under **Other** click **Account Connections**.

## Creating a Google Drive source

To add a Google Drive file or folder as a project source:

1. Go to the project **Sources** page

![](http://stencila.github.io/hub/manager/snaps/project-sources-menu-item.png)

2. Press the **New** button and select **Google Drive**

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-button.png)

3. Enter the **File or Folder URL** or **id**

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-googledrive-url.png)

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-googledrive-repo.png)

4. Add the **Path** that you want to link it to within the project (what you want to name your source) and **Create Source**

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-path-field.png)

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-create-button.png)

## Your Google Drive source

Your Google Drive source is stores in your project's working directory as a folder. Within the folder will be all the files which have automatically "pulled".

Currently, only **non-native** files such as images, pdfs and data files get pulled down. Any **google formats** such as Google Docs, Google Sheets, Google Slides etc do not currently get pulled down. You can pull [Google Docs](./google-docs.md) individually. We are working to correct this issue https://github.com/stencila/hub/issues/771.

