# Contributing to mdzk

Welcome! I'm glad you want to help improve mdzk. This guide will help you through the process.

### If you find a bug or have a feature request, just follow these steps:

1. Search through the [issues](https://github.com/mdzk-rs/mdzk/issues) to see if anyone has had the idea already. 
 
    - If you find a closed issue: Congrats! Your adventure is done. If this is in reference to a bug that is still present on you part, first check if the fix is actually released. If it is, consider reopening the issue and adding any additional information you might have.
    - If you find an open issue: Read through the thread and consider adding any additional information you might have. Proceed to step 3.
    - If no issue is present: Proceed to step 2.

2. Open an issue describing the bug/feature. We have a template that will help you along the way.
    
    **Note:** It's always better to discuss a bit before jumping head-first into the code!
    
3. Try writing your own fix or implementation. Fork the repo and do any changes you deem fit.
    
    GitHub provides a [great guide to opening pull requests](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests). There are also a couple of styling notes to keep in mind:
    
    - We use [Conventional Commits](https://www.conventionalcommits.org). You will be asked to change any commit messages that do not follow this convention.
    - Run `cargo fmt` before commiting, to ensure your code is properly formatted.

4. Run `cargo test` and ensure everything passes. Consider adding your own unit tests where relevant.
5. Open a PR. The template will guide you through the required steps, so if you forgot something, go back and change it before submitting.

We will review your PR as soon as we have the opportunity! If we need any changes done, we'll notify you. When everything is done, your code will finally get merged 🥇
