# Guide to Contributing

First off, thanks for taking the time to contribute! ❤️

All types of contributions are encouraged and valued. See the [Table of Contents](#table-of-contents) for different ways to help and details about how this project handles them. Please make sure to read the relevant section before making your contribution. It will make it a lot easier for us maintainers and smooth out the experience for all involved. The community looks forward to your contributions. 🎉

> And if you like the project, but just don't have time to contribute, that's fine. There are other easy ways to support the project and show your appreciation, which we would also be very happy about:
>
> - Star the project
> - Tweet about it
> - Refer this project in your project's README
> - Mention the project at local meetups and tell your friends/colleagues

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [I Have a Question](#i-have-a-question)
- [I Want To Contribute](#i-want-to-contribute)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Enhancements](#suggesting-enhancements)

_Note:_ This guide is based on the [contributing.md](https://contributing.md/).

## Code of Conduct

This project and everyone participating in it is governed by our
[Code of Conduct](blob/main/CODE_OF_CONDUCT.md).
By participating, you are expected to uphold this code. Please report unacceptable behavior
to [hello@qlty.sh](hello@qlty.sh).

## I Have a Question

> If you want to ask a question, we assume that you have read the available [Documentation](https://docs.qlty.sh).

Before you ask a question, it is best to search for existing [Issues](/issues) and [Discussions](/discussions) that might help you. In case you still need clarification, you can write your question as a reply. It is also advisable to search the internet for answers first.

If you then still feel the need to ask a question and need clarification, we recommend the following:

- Open a [Q&A discussion](https://github.com/orgs/qltysh/discussions/categories/q-a).
- Provide as much context as you can about what you're running into.
- Please fill out the form completely

The community will be able to reply right away, and the Qlty team aims to reply within one week.

## I Want To Contribute

> ### Legal Notice
>
> When contributing to this project, you must agree that you have authored 100% of the content, that you have the necessary rights to the content and that the content you contribute may be provided under the project license.
>
> Contributions require agreeing to our [Contributor License Agreement](https://gist.github.com/brynary/00d59e41ffd852636a2f8a8f5f5aa69b) (CLA).

### Reporting Bugs

#### Before submitting a bug report

A good bug report shouldn't leave others needing to chase you up for more information. Therefore, we ask you to investigate carefully, collect information and describe the issue in detail in your report. Please complete the following steps in advance to help us fix any potential bug as fast as possible.

- Make sure that you are using the latest version of the CLI and any sources and plugins
- Determine if your bug is really a bug and not an error on your side e.g. using incompatible environment components/versions
  - Make sure that you have read the [documentation](https://docs.qlty.sh).
  - If you are looking for support, you might want to check [this section](#i-have-a-question)).
- To see if other users have experienced (and potentially already solved) the same issue you are having, check if there is not already a bug report existing for your bug or error in the GitHub [Issues](/issues) or [Discussions](https://github.com/orgs/qltysh/discussions).
- Collect information about the bug:
  - Stack trace (Traceback)
  - OS, Platform and Version (Windows, Linux, macOS, x86, ARM)
  - Version of the interpreter, compiler, SDK, runtime environment, package manager, depending on what seems relevant.
  - Possibly your input and the output
  - Can you reliably reproduce the issue? And can you also reproduce it with older versions?

#### How do I submit a good bug report?

> You must never report security related issues, vulnerabilities or bugs including sensitive information to the issue tracker, or elsewhere in public. Instead sensitive bugs must be sent by email to [security@qlty.sh](mailto:security@qlty.sh).

We use GitHub issues to track bugs and errors. If you run into an issue with the project:

- Open an [Issue](/issues/new) using the Bug Report template.
- Explain the behavior you would expect and the actual behavior.
- Please provide as much context as possible and describe the _reproduction steps_ that someone else can follow to recreate the issue on their own. This usually includes your code. For good bug reports you should isolate the problem and create a reduced test case.
- Provide the information you collected in the previous section.

Once it's filed:

- The project team will label the issue accordingly.
- A team member will try to reproduce the issue with your provided steps. If there are no reproduction steps or no obvious way to reproduce the issue, the team will ask you for those steps. Bugs without reproductions will not be addressed until they are reproduced.
- If the team is able to reproduce the issue, the issue will be left to be [implemented by someone](#your-first-code-contribution).

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion, including completely new features and minor improvements to existing functionality. Following these guidelines will help maintainers and the community to understand your suggestion and find related suggestions.

#### Before submitting an enhancement

- Make sure that you are using the latest version of the CLI and all sources and plugins
- Read the [documentation](https://docs.qlty.sh) carefully and find out if the functionality is already covered, maybe by an individual configuration.
- Perform searches of [issues](/issues) and [discussions](https://github.com/orgs/qltysh/discussions) to see if the enhancement has already been suggested. If it has, add a comment instead of opening a new issue or discussion.
- Find out whether your idea fits with the scope and aims of the project. It's up to you to make a strong case to convince the project's developers of the merits of this feature. Keep in mind that we want features that will be useful to the majority of our users and not just a small subset.

#### How do I submit a good enhancement suggestion?

Enhancement suggestions are tracked as GitHub Discussions in the [Feedback category](https://github.com/orgs/qltysh/discussions/categories/feedback).

- Use a **clear and descriptive title** for the issue to identify the suggestion.
- Provide a **step-by-step description of the suggested enhancement** in as many details as possible.
- **Describe the current behavior** and **explain which behavior you expected to see instead** and why. At this point you can also tell which alternatives do not work for you.
- You may want to **include screenshots and animated GIFs** which help you demonstrate the steps or point out the part which the suggestion is related to. You can use [this tool](https://www.cockos.com/licecap/) to record GIFs on macOS and Windows, and [this tool](https://github.com/colinkeenan/silentcast) or [this tool](https://github.com/GNOME/byzanz) on Linux.
- **Explain why this enhancement would be useful** to most Qlty users. You may also want to point out the other projects that solved it better and which could serve as inspiration.

## Triage Process

### Step 1. Validate

In order to focus our efforts were they'll have the largest impact, we require that submissions are sent through the right channel based on their content, and adhere to our form templates.

If we receive reports in the wrong channels or without the necessary details, we will reply and close them in order to keep things tidy.

### Step 2. Respond

_Target_: One week

We aim to provide an initial response from a Qlty team member within one week by responding to each Issue, Discussion, or Pull Request with one of the following actions:

1. Requesting more information, if needed
2. Making and communicating a decision about if or how we would like to proceed
3. Closing the ticket

### Step 3. Resolve

_Target:_ None

Once a ticket has sufficient information to proceed to implementation, it is up to the Qlty team to decide if and when it will be worked on. The team will communicate with customers via GitHub when updates are available.

**GitHub is the source of truth for the customer.**
