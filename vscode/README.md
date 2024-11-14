# Stencila VSCode Extension

This extension provides an interface to [Stencila](https://stencila.io), a tool to help scientists and researchers to develop rich documents utilizing a structured schema. Stencila bring semantic horsepower and flexible executable documents with structure 🏗️ and power 💪🏼 driven by the [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema).

> ⚠️ This is an early release of the extension. We'd love for you to try it out but expect bugs and missing docs.
> Please report any issues or ideas on our Github repo, or join us on our Discord server!

[Discord](https://discord.gg/GADr6Jv) | [Code of Conduct](https://github.com/stencila/stencila/blob/main/CODE_OF_CONDUCT.md) | [Contributing](https://github.com/stencila/stencila/blob/main/vscode/CONTRIBUTING.md) | [Contributors](https://github.com/stencila/stencila#-contributors) | [License](https://github.com/stencila/stencila/blob/main/vscode/LICENSE)

## About Stencila

Stencila is an open-source toolkit focused on enhancing **documents with scientific intelligence**. It enables users to create interactive, data-rich documents that integrate seamlessly with code, data, and text, supporting reproducible research and collaboration. With Stencila’s VS Code extension, you can build smart documents that leverage AI tools, ensure transparency, and support interoperability across languages and platforms, all within your existing workflow.


![signing in with stencila](./images/createDemo.gif)


## Enabling LLM support

Various back-ends can be used to provide LLM support in Stencila documents.  You must have at least one of these set up to use AI commands. Stencila will automatically select and use one of these based on the order below:

### Option 1: Stencila Cloud

**Stencila Cloud** is the easiest way to get up and running with a variety of high performance online models with just one account.  You can easily select which model to use for each prompt or leave it blank and it will select the best one for you via Stencila Router.

To use Stencila Cloud, click **Sign In** from the Stencila extension menu, and follow the steps in your web browser to create a new account or sign into an existing account.  You will receive a number of free credits to try out Stencila Cloud and after that you can sign up for a subscription plan to continue usage.

![signing in with stencila](./images/signInDemo.gif)

### Option 2: Use your own API keys

If you already have your own **API keys** for one of the supported LLM services, you can enter your keys in the Stencila extension menu using **Set Secret**.  You will need to log into your own LLM services dashboard to retrieve your API key.

### Option 3: Use a locally running model

If you have a locally running **Ollama** model already on your computer as a background process, Stencila will automatically detect and use it.  See the Ollama [website](https://ollama.com/) for more information on how to install and run it.

## Walkthroughs

The included walkthroughs cover the basics of creating structured, responsive documents, and introduce key concepts in Stencila’s syntax for adding styles, layouts, and components.

Each walkthrough includes examples and explanations, allowing users to explore and practice Stencila commands in a guided environment. Whether you're new to Stencila or looking to enhance your skills, these resources provide a practical foundation to get started with creating scientific and data-driven documents. They are updated alongside all the features within Stencila.

You can find these walkthroughs under the Stencila menu.

![signing in with stencila](./images/walkThroughDemo.gif)


