# Draft a changelog entry for an upcoming release

You are drafting the CHANGELOG.md entry for an upcoming release of this
repository. The workflow invoking you provides three values:

- `VERSION`: the version being released, without a leading "v"
- `LAST_TAG`: the git tag of the most recent release
- `DATE`: today's date (YYYY-MM-DD)

Follow these steps:

1. Use `git log ${LAST_TAG}..HEAD --oneline --first-parent` to list the
   changes merged since the last release. (If a change was not made through a
   pull request, you can safely skip it.)
2. For each merged pull request, retrieve the PR title and description from
   GitHub for additional context using `gh pr view`.
3. Ignore changes which are chores, docs only, CI changes, tests only,
   style/formatting changes, build pipeline changes, and routine dependency
   bumps that don't change the CLI's behavior. The CHANGELOG.md only contains
   changes which are externally observable new features, enhancements, and
   fixes. (A dependency update that fixes a security vulnerability in the
   published CLI counts as a fix.)
4. Add exactly one new section to the top of CHANGELOG.md, directly below the
   `# Changelog` heading: `## v${VERSION} (${DATE})`. Follow the existing
   format of the file — group changes under `### New`, `### Improved`, and
   `### Fixed` subheadings when there is more than one kind of change, and
   reference PR numbers like `(#2762)`.
5. If there are no externally observable changes, still add the section with a
   single brief bullet summarizing the most significant internal change.
6. If CHANGELOG.md contains an `## Unreleased` section, fold its bullets into
   the new version's section and remove the `## Unreleased` heading, so the
   same changes are not listed twice.
7. If a PR author is not a member of the team, include a shout out thanking
   them. (Team members are @noahd1, @lsegal, @brynary, @marschattha,
   @davehenton, @laura-mlg. They don't need shout outs.)

Rules:

- Only edit CHANGELOG.md, and only add the single new section. Do not reword
  or reorganize existing entries.
- Do not commit, push, or open pull requests; the workflow handles that.
