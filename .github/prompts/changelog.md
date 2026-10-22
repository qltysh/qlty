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
   GitHub for additional context using `gh pr view`. Use the description to
   understand the change, not as text to copy. PR descriptions explain why and
   how for reviewers; the changelog tells users what is different for them.
3. Ignore changes which are chores, docs only, CI changes, tests only,
   style/formatting changes, build pipeline changes, and routine dependency
   bumps that don't change the CLI's behavior. The CHANGELOG.md only contains
   changes which are externally observable new features, enhancements, and
   fixes. (A dependency update that fixes a security vulnerability in the
   published CLI counts as a fix.)
4. Add exactly one new section to the top of CHANGELOG.md, directly below the
   `# Changelog` heading: `## v${VERSION} (${DATE})`. Follow the existing
   format of the file — group changes under `### New`, `### Improved`, and
   `### Fixed` subheadings when there is more than one kind of change.
5. Write each entry as one bullet:
   - One sentence. Add a second only when the first can't stand alone.
   - Start with a verb (Add, Fix, Show, Parse, Require) and say what changed
     for the user.
   - Name the user-facing surface exactly: command, flag, `qlty.toml` key, or
     plugin, in backticks.
   - Always end with the number of the pull request that made the change, in
     parentheses: `... in help output (#2943)`. Omit it only when the change
     has no pull request. No period after the number. Never drop the number
     to make a bullet shorter; shorten the sentence instead.
   - For a fix, say what now works. Add the old symptom only if users would
     recognize it by its error message.
6. Leave these out of every bullet:
   - The reason the change was made or who benefits from it ("so that docs
     generation can...", "to match `qlty init`").
   - Internal function, struct, or file names.
   - Lists of everything a change touched, such as each flag value or each
     help line. Name the headline and stop.
   - Statements that something is unchanged ("plugin selection behavior is
     unchanged").
   - Numbers from the incident that prompted the fix.
7. If a change is a hidden or internal command, write one short bullet that
   says what it does and nothing more.
8. If there are no externally observable changes, still add the section with a
   single brief bullet summarizing the most significant internal change.
9. If CHANGELOG.md contains an `## Unreleased` section, fold its bullets into
   the new version's section and remove the `## Unreleased` heading, so the
   same changes are not listed twice.
10. If a PR author is not a member of the team, include a shout out thanking
    them, after the PR number: `(#2750) — thanks @cussiol!`. (Team members are
    @noahd1, @lsegal, @brynary, @marschattha, @davehenton, @laura-mlg,
    @fhwang. They don't need shout outs.)

Examples of the target style:

- Resolve Stylelint `extends` and `plugins` against the tool install
  directory (#2757)
- Show the `--trigger` flag in `qlty check --help` and describe it in
  `qlty fmt --help` (#2943)
- Add a hidden `qlty cli-reference` command that prints the CLI command tree
  as JSON (#2940)
- Parse `note` as a level in `qlty.toml` triage and ignore config (#2922)

Rules:

- Only edit CHANGELOG.md, and only add the single new section. Do not reword
  or reorganize existing entries.
- Do not commit, push, or open pull requests; the workflow handles that.
- Before finishing, reread each bullet and cut it to the shortest version that
  still tells a user what changed.
