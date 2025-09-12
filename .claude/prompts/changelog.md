We need to update the CHANGELOG.md file to make sure everything important that has recently been merged into the `main` branch is represented.

First, use `gh release list` to see all the recent releases. Make sure every recent release is represented as a subheading in CHANGELOG.md.

Then, use `git log` to see which pull requests were merged into each release. (If a change was not done through a pull request, you can safely skip it.)

For each merged pull request, retrieve the PR title and description from GitHub for additional context using `gh pr view`.

Ignore changes which are chores, docs only, CI changes, tests only, style/formatting changes, and build pipeline changes. The CHANGELOG.md only contains changes which are externally observable new features, enhancements, and fixes.

Follow the existing format of the CHANGELOG.md file. Organize changes underneath headers for the version number and date.

Check to see if there are any chnages in `main` which are not in the most recent release by inspecting the Git log since the most recent release tag versus the latest commit on main. If there are changes in main which aren't released yet, check if you are working in a release branch. A release branch is named by convention release-${version}. If you are in a release branch, name the header for the version indicated by the branch. Otherwise, make an "Unreleased" header at the top of the CHANGELOG.md for those.

Finally, if a PR author is not a member of the team, be sure to include a shout out. (Team members are @noahd1, @lsegal, @brynary, @marschattha, @davehenton, @laura-mlg. They don't need shout outs.)

Finally, run `qlty fmt` to auto-format the results.
