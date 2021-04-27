---
title: Google Drive Sources
description: Add a Google Drive to your Stencila project
relatedArticles:
  - ../projects/manage-project-sources.md
authors:
  - 'Colette Doughty <colette@stenci.la> (https://stenci.la)'
published: true
---

A Google Drive source lets you pull files from your Google Drive into your project so that you can convert files to other formats (e.g. a Word document), use them in compute sessions (e.g. CSV files) or publish. This allows you to take full advantage of Google Drive's many benefits such a cloud-based storage solution and real-time collaboration which you might already be using with your colleagues and to easily pull this information into your Stencila project at any time.

## Before you start

Before you start you will need to make sure your google account is connected to your Stencila account to be able to access the required documents. To check, go to your account **Settings** in the top right corner of your screen, scroll to the bottom of the page and under **Other** click **Account Connections**.

## Creating a Google Drive source

To add a Google Drive file or folder as a project source:

- Go to the project **Sources** page

![](http://stencila.github.io/hub/manager/snaps/project-sources-menu-item.png)

- Press the **New** button and select **Google Drive**

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-button.png)

- Enter the **File or Folder URL**

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-googledrive-url.png)

- Or **id**

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-googledrive-id.png)

- Add the **Path** that you want to link it to within the project (what you want to name your source)

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-path-field.png)

![](http://stencila.github.io/hub/manager/snaps/project-sources-new-create-button.png)

## Your Google Drive source

Your Google Drive source is stores in your project's working directory as a folder. Within the folder will be all the files which have automatically "pulled".

Currently, only **non-native** files such as images, pdfs and data files get pulled down. Any **google formats** such as Google Docs, Google Sheets, Google Slides etc do not currently get pulled down. You can pull [Google Docs](./google-docs.md) individually. We are working to correct this issue https://github.com/stencila/hub/issues/771.
