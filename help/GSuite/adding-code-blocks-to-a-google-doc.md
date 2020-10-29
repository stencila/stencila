---
title: Adding code blocks to a Google Doc
description: How to execute and insert code into your Google document
id: 4170136
relatedArticles:
  - ../hub/sources/google-docs.md
authors:
  - 'Colette Doughty <colette@stenci.la> (https://stenci.la)'
---

## Overview

To be able to execute and insert code into your Google document the basic workflow consists of:
- Create a Block, either a Code or an Equation block
- Run the Block in the sidebar
- Insert the results into the document.

## Before you start

We presume you have already:
- Signed up to **Stencila**. If you haven't signed up yet, see [creating an account](../hub/getting-started.md). 

- Installed the **Stencila for Google Docs** add on and have opened the side bar. If you haven't installed it yet, see ARTICLE TO COME.

![](https://i.imgur.com/snR8gbW.png)

## Create a project

In order to have the right library versions and make use of data files, you need to link your Google Doc to an exsisting Stencila Hub project or create a new project. 

![](https://i.imgur.com/3oXtlmd.png)

## Start creating

Now that the document is linked as a project source, you can create either a Code Block, or an Equation Block for writing LaTeX equations.
Click on Code to create a new block. 

![](https://i.imgur.com/syubvng.png)

This view is split into three parts. At the top is the Toolbar, from here you can:
- Run the code block
- insert it into the document
- discard changes
The insert into document and discard change buttons will appear after you’ve run the code block.

## Code Editor

You can configure the programming language and type code in the editor.

![](https://i.imgur.com/W1TF4Yx.png)

Insert image

## Output preview

At the bottom is the Output Preview. Results from running your code block will be shown here. Inserting images in the document is relative slow, so this lets you quickly iterate on your visualization.

Insert image

## Let's write some code

First select the programming language from the dropdown. 
Then enter some code in the top of the code editor box. (Here’s a simple R code for you to enter: **head(mtcars)**)
When ready, click the ▶️ (Play/Run) button at the right edge of the blue toolbar.

This is the preview of running your code, it’s not part of the document yet.

Insert image

When ready, click the Insert into Document (Save) button in the toolbar.

You are now embedding the source code into the image, and inserting it into the document. This keeps everything nice and reproducible ☺️

## Next steps?
WHAT COMES NEXT? is it how to publish your project?
