# Contributing to Rusty AutoClicker

## Table of Contents

- [Asking Support Questions](#asking-support-questions)
- [Reporting Issues](#reporting-issues)
- [Code Contribution](#code-contribution)
- [Submitting Patches](#submitting-patches)
  - [Code Contribution Guidelines](#code-contribution-guidelines)
  - [Git Commit Message Guidelines](#git-commit-message-guidelines)

## Asking Support Questions

We have a [discussion forum](https://github.com/MrTanoshii/rusty-autoclicker/discussions) where users and developers can ask questions.
Please don't use the GitHub issue tracker to ask questions.

## Reporting Issues

If you believe you have found a defect in Rusty AutoClicker or its documentation, use
the GitHub issue tracker to report the problem to the maintainers. If you're not sure if it's a bug or not,
start by asking in the [discussion forum](https://github.com/MrTanoshii/rusty-autoclicker/discussions).
When reporting the issue, please provide the version of Rusty AutoClicker in use and your operating system.

- [Rusty AutoClicker Issues · MrTanoshii/rusty-autoclicker](https://github.com/MrTanoshii/rusty-autoclicker/issues)

## Code Contribution

New functionality must:

- be useful to many.
- not bloat the binary.
- retain compatibility with Windows, Linux and macOS.
- close or update an open [issue](https://github.com/MrTanoshii/rusty-autoclicker/issues)

If it is of some complexity, the contributor is expected to maintain and support the new feature in the future (answer questions on the forum, fix any bugs etc.).

It is recommended to open up a discussion on the [discussion Forum](https://github.com/MrTanoshii/rusty-autoclicker/discussions) to get feedback on your idea before you begin.

**Bug fixes are, of course, always welcome.**

## Submitting Patches

The Rusty AutoClicker project welcomes all contributors and contributions regardless of skill or experience level. If you are interested in helping with the project, we will help you with your contribution.

### Code Contribution Guidelines

Because we want to create the best possible product for our users and the best contribution experience for our developers, we have a set of guidelines which ensure that all contributions are acceptable. The guidelines are not intended as a filter or barrier to participation. If you are unfamiliar with the contribution process, the Rusty AutoClicker team will help you and teach you how to bring your contribution in accordance with the guidelines.

To make the contribution process as seamless as possible, we ask for the following:

- Fork the project and make your changes. We encourage pull requests to allow for review and discussion of code changes.
- When you’re ready to create a pull request, be sure to:
  - Run `cargo fmt`.
  - Add documentation if you are adding new features or changing functionality.
  - Follow the **Git Commit Message Guidelines** below.

### Git Commit Message Guidelines

This [article](https://www.freecodecamp.org/news/how-to-write-better-git-commit-messages/) is a good resource for learning how to write good commit messages,
the most important part being that each commit message should have a title/subject in imperative mood starting with no trailing period:
_"feat: add double left click"_, **NOT** _"Adding double left click."_
