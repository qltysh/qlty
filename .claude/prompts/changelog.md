We need to update the CHANGELOG.md file to make sure everything important that has recently been merged into the `main` branch is represented.

First, review the Git log to see what pull requests were merged. (If a change was not done through a pull request, you can safely skip it.)

For each merged pull request, retrieve the PR title and description from GitHub for additional context using `gh pr view`.

Ignore changes which are chores, docs only, CI changes, tests only, style/formatting changes, and build pipeline changes. The CHANGELOG.md only contains changes which are externally observable new features, enhancements, and fixes.

Follow the existing format of the CHANGELOG.md file. Organize changes underneath headers for the version number and date. If changes are not yet in a released version, make an "Unreleased" header at the top of the CHANGELOG.md for those.

Finally, if a PR author is not a member of the team, be sure to give them a shout out. (Team members are @noahd1, @lsegal, @brynary, @marschattha, @davehenton, @laura-mlg. They don't need shout outs.)

When you're done, if there are any changes to CHANGELOG.md, open a pull request.