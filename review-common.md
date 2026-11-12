<!-- Shared Qlty review rules. Keep this copy identical to the original, apart from this line. --> <!-- markdownlint-disable-file MD041 -->

## Shared rules

These apply in every repo that includes this file.

### What Important means

Reserve Important for a likely bug, a risk that could break production or a deploy, a security problem, a design choice that will be costly to reverse, or a clear violation of a written rule in new code. The written rules are the code rules in the CLAUDE.md files that cover the changed files, including `.claude/CLAUDE.md`, and the files in `.claude/rules/`. Read them. A CLAUDE.md below the repo root, other than `.claude/CLAUDE.md`, covers only the files under its own directory, and a rules file with `paths:` covers only the files they match. Process guidance, such as how to plan or open a PR, and the review instructions themselves are not written rules for this purpose. Important findings block the merge until someone fixes them or a reviewer overrides them, so be sure before you mark one. Another section of REVIEW.md can set a different severity for the changes it covers.

Style, naming, comments and wording, and formatting CI doesn't check, are Nit at most, even where a written rule covers them, unless another section of REVIEW.md says otherwise. A change that breaks existing callers or behavior is not a style issue, whatever form it takes.

### Missing tests

A missing test is Important for code that reads or writes a database, migrations, authentication, authorization and billing. A repo's REVIEW.md can name more cases. Elsewhere a missing test is Nit at most, even where a written rule asks for tests.

### Parsing

Where a parser exists for a format, such as URLs, paths, JSON, HTML or Markdown, code should use it instead of a regex or string matching. Check each regex and string match the change adds or changes, including a pattern built as a string, against this section and the repo's written rules. For regexes, and for string checks on those formats, this section sets the severity, not the rule about written-rule violations.

- A URL check that decides where a credential is sent or where to redirect is Important unless it compares the parsed origin for equality with an expected value or an allowlist entry, not by prefix, suffix or substring.
- Any other regex or string check on such a format is Important when you can name a realistic input it gets wrong that the parser would get right. Name that input. A search over unstructured text, such as blanking ids in a recorded body, is fine.
- User input meant as literal text, placed in a regex without escaping or inside a character class, is Important. A pattern the customer or user writes, such as a glob, falls under the next bullet.
- A new place that runs a regex or glob a customer or user writes is Important. Say where it runs, so a person can decide how to bound its run time.
- Any other departure from this section or the repo's written parsing rules is a Nit, such as a regex a string method would replace. A check that you can show causes a bug or a security problem is still Important.

### Name a shared cause

When problems you report share one cause in the change's approach, say so in the most severe finding, list the others, and suggest an approach that avoids them all. Do this only when changing the approach would remove every problem you list. Don't post it as a finding of its own or raise its severity for it. The suggestion is advisory: fixing the problems themselves resolves the findings.

### Cap the nits

Report at most three Nits, most consequential first. Nits from a REVIEW.md section with its own cap don't count toward these three.

### Do not report

- Anything this repo's CI already enforces, such as lint or formatting
- Lockfiles
- Generated, vendored and minified files, unless a REVIEW.md section asks you to check them. Migrations, API specs and text test snapshots are reviewed even when a tool produced them. Recorded test fixtures are not generated for this rule: check them for secrets and customer data.
