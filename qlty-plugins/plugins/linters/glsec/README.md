# glsec

[glsec](https://github.com/glsec/glsec) is a security linter for GitLab CI/CD pipelines. It reports mutable images and includes, script injection through branch names and commit messages, secrets in variables, unprotected deployments and similar issues in `.gitlab-ci.yml`.

## Enabling glsec

Enabling with the `qlty` CLI:

```bash
qlty plugins enable glsec
```

Or by editing `qlty.toml`:

```toml
# Always use the latest version
[[plugin]]
name = "glsec"

# OR pin to a specific version
[[plugin]]
name = "glsec"
version = "X.Y.Z"
```

## Auto-enabling

glsec will be automatically enabled by `qlty init` when a GitLab CI/CD pipeline file is present.

## Languages and file types

glsec analyzes: GitLab CI/CD pipeline files (`.gitlab-ci.yml` and files named `*.gitlab-ci.yml`).

## Configuration files

- [`.glsec.yml`](https://github.com/glsec/glsec#exit-codes)

To keep your project tidy, you can move configuration files into `.qlty/configs` and Qlty will find and use them when running glsec.

## Troubleshooting

**glsec reports nothing for jobs that come from an `include:`.**
glsec follows local child pipelines (`trigger: include:` with a local path), but it does not fetch remote, `project:`, `template:` or `component:` includes. Jobs defined there are not checked unless their files are in the repository and match the file types above.

**A finding is not reported although the rule matches.**
glsec honours `# glsec:ignore GLxxx` comments, a `.glsec-ignore` baseline and the settings in `.glsec.yml`. Check those files first. Each rule's page on [glsec.github.io](https://glsec.github.io/rules/) lists what it reports and how to fix it.

## Links

- [glsec on GitHub](https://github.com/glsec/glsec)
- [glsec rule reference](https://glsec.github.io/rules/)
- [glsec plugin definition](https://github.com/qltysh/qlty/tree/main/qlty-plugins/plugins/linters/glsec)
- [glsec releases](https://github.com/glsec/glsec/releases)
- [Qlty's open source plugin definitions](https://github.com/qltysh/qlty/tree/main/qlty-plugins/plugins/linters)

## License

glsec is licensed under the [Apache License 2.0](https://github.com/glsec/glsec/blob/main/LICENSE).
