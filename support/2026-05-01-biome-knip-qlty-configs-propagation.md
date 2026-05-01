# Escalation: biome and knip do not honor `.qlty/configs/` configs

**Date:** 2026-05-01
**Workspace:** [Ben's workspace]
**User:** Ben Carey
**User ID:** [from Plain]
**Link to Conversation:** [Slack permalink]
**ARR:** [from billing]

---

## Description

Configs placed in `.qlty/configs/` are not honored by the biome and
knip plugins, despite docs stating this is the canonical location and
that configs are "automatically moved to the correct location" at
analysis time.

The current mechanism for all plugins is a symlink at the workspace
root (`executor.rs:350–370` + `staging_area.rs:244–252`), placing e.g.
`workspace_root/biome.json -> .qlty/configs/biome.json`. This works
for tools that run from `InvocationDirectoryType::Root` *and* follow
symlinks. It fails in two cases relevant here:

1. **Knip** runs with `InvocationDirectoryType::TargetDirectory` — its
   CWD is the directory containing `package.json`, not workspace root.
   The symlink at workspace root is not visible to knip's config
   discovery. Affects any monorepo / subdirectory `package.json` setup.
   Confirmed bug.

2. **Biome** runs from workspace root, so the symlink is in its CWD.
   In theory biome's implicit config discovery should find it, but in
   practice configs at `.qlty/configs/biome.json` are ignored. Likely
   cause: biome not following symlinks. Either way, qlty's contract
   with users (per docs) is broken.

---

## Steps to Reproduce

1. Place valid `biome.json` and `knip.json` at `.qlty/configs/` with
   any rule-disabling configuration
2. Run `qlty check --all --filter=biome` — disabled rules still fire
3. Run `qlty check --all --filter=knip` (in a repo where `package.json`
   lives in a subdirectory) — disabled rules still fire
4. Direct invocation (`npm run lint`) honors the same configs

---

## Expected Behavior

Configs in `.qlty/configs/` should be honored for all plugins, as
documented at https://docs.qlty.sh/analysis-configuration.

---

## Workaround

- **biome:** place `biome.json` at repo root (real file, not symlinked)
- **knip:** place `knip.json` next to `package.json` (not at repo root,
  not in `.qlty/configs/`)

Customer has been advised.

---

## Suggested Fix

Follow the PHPStan / ESLint v9+ pattern. Add `${config_script}` with
explicit `--config-path` (biome) / `--config` (knip) flag, and set
`copy_configs_into_tool_install = true` on both drivers. This copies
the config into `tool.directory()` and makes `${config_file}` resolve
to that absolute path, which is independent of CWD and immune to
symlink-handling differences across tools.

**`qlty-plugins/plugins/linters/knip/plugin.toml`:**
```diff
 [plugins.definitions.knip.drivers.lint]
-script = "knip --no-progress --reporter json"
+script = "knip --no-progress --reporter json ${config_script}"
+config_script = "--config ${config_file}"
+copy_configs_into_tool_install = true
 target = { type = "parent_with", path = "package.json" }
 runs_from = { type = "target_directory" }
```

**`qlty-plugins/plugins/linters/biome/plugin.toml`:**
```diff
 [plugins.definitions.biome.drivers.lint]
-script = "biome lint --reporter=json ${target}"
+script = "biome lint --reporter=json ${config_script} ${target}"
+config_script = "--config-path ${config_file}"
+copy_configs_into_tool_install = true

 [plugins.definitions.biome.drivers.format]
-script = "biome format --write ${target}"
+script = "biome format --write ${config_script} ${target}"
+config_script = "--config-path ${config_file}"
+copy_configs_into_tool_install = true
```

`replace_config_script()` (`invocation_script.rs:43–53`) replaces
`${config_script}` with the empty string when no config exists, so
both tools continue to work for users with no config file.

---

## Broader Question for Engineering

The single-symlink-at-workspace-root mechanism is silently broken for
any plugin running from `TargetDirectory`. Knip is the only plugin
currently using that mode, but it's worth considering whether
Mechanism A should also place the file at the actual invocation
directory, to prevent this class of bug from recurring as new plugins
are added.

---

## Additional Context

- **CLI version:** [pending from Ben]
- **Affected scope:** any user storing biome or knip configs in
  `.qlty/configs/` per docs guidance; knip case is universal for
  monorepo/subdir setups
- **Reporter:** performed independent investigation and identified the
  general area before escalation
