function refactorPrompt(row, week, context, weekLabel) {
  const current = row.after.length > 0;
  const target = (current ? row.after : row.before)[0];
  const grouped = row.before.length > 1 || row.after.length > 1;
  const revision = current ? week.commit : week.before_commit;
  const sourceUrl = `${context.repository}/blob/${revision}/${target.path.split('/').map(encodeURIComponent).join('/')}`;
  const member = file => ({
    path: file.path,
    status: file.passed ? 'PASS' : 'FAIL',
    score: file.score,
    language: file.language,
    loc: file.code_lines,
    cyclomatic_complexity: file.cyclomatic,
    mass: file.mass,
    analyzed_source_scope: file.source_scope,
  });
  const explanation = row.explanation;
  const questions = {};
  for (const factor of explanation?.factors ?? []) {
    if (context.questions[factor.id]) questions[factor.name] = context.questions[factor.id];
  }
  const data = {
    repository: context.repository,
    interval: context.interval || 'week',
    period: weekLabel,
    change_type: row.kind,
    revisions: {before: week.before_commit, after: week.commit},
    diff_url: `${context.repository}/compare/${week.before_commit}...${week.commit}#diff-${row.diff_anchor}`,
    primary_file: {...member(target), snapshot: current ? 'after' : 'before', revision, source_url: sourceUrl},
    scoring: {
      range: [1, 10],
      higher_is_better: true,
      pass_threshold: context.threshold_score,
      pass_rule: 'Score strictly greater than the threshold; use the saved status, not a rounded score.',
      meaning: context.model.score_meaning,
      model_id: context.model.id,
      model_sha256: context.model.sha256,
      score_scale: context.model.score_scale,
      measurement_policy: context.model.measurement_policy,
    },
    comparison: {
      scope: grouped ? 'Group of files; scores and factor impacts are weighted by file mass.' : 'Single file',
      before: row.before.map(member),
      after: row.after.map(member),
      before_score: row.before_score,
      after_score: row.after_score,
      before_mass: row.before_mass,
      after_mass: row.after_mass,
      mass_definition: 'Cyclomatic complexity × square root of LOC',
      period_impact: row.contribution,
      period_impact_meaning: 'Contribution to the project result for the selected period; distinct from the file score and its factor impacts.',
    },
    score_explanation: explanation ? {
      basis: explanation.mode === 'score'
        ? 'Score relative to the model reference; not a before-and-after change.'
        : 'Before-and-after score change; factor impacts sum to that change.',
      ...(explanation.mode === 'score'
        ? {reference_score: explanation.reference_score}
        : {score_change: explanation.score_change}),
      leading_factors: explanation.factors.map(factor => ({
        name: factor.name,
        impact_in_score_points: factor.change,
        measurements: factor.evidence,
      })),
      other_factors_impact: explanation.other,
      interpretation: 'Positive impact raises the score; negative impact lowers it. These are model attributions, not proof of a code defect or estimates of refactoring benefit.',
    } : null,
    ai_assessment_questions: questions,
  };
  const lines = [
    'This project was analyzed by qlty slop-one. Review one primary file and recommend refactorings that improve maintainability for human software engineers.',
    '',
    `Primary file: ${target.path}`,
    `Project: ${context.repository}`,
    `Analysis snapshot: ${weekLabel}`,
    '',
    'All report data for this review is included below; no separate qlty slop-one report or data files are needed. Inspect the actual source code in the project.',
    '',
    'Use the current checkout as the working context. The analysis is a historical snapshot: first check whether this file still exists and whether the findings still apply. Use the supplied revisions and diff for context without resetting the checkout.',
  ];
  if (row.kind === 'Removed') {
    lines.push('This file was removed in the selected period. Its score and measurements describe the earlier revision. Inspect any replacement or surviving code; do not restore deleted code merely to address historical findings. It is valid to conclude that no further change is needed.');
  } else if (row.kind === 'Renamed') {
    lines.push(`The file was renamed from ${row.before[0].path}. Start with the new path and follow later moves if necessary.`);
  } else if (row.kind === 'Added') {
    lines.push('This file was added in the selected period, so there is no before score. Its factor impacts explain the score relative to a model reference, not a decline from an earlier version.');
  }
  if (grouped) {
    lines.push('This entry represents a split, merge, or reorganization. The primary_file record gives its own score; comparison scores and factor measurements cover the full group. Do not attribute every group finding to the primary file. Inspect the listed related files when judging its responsibilities and boundaries.');
  }
  lines.push(
    '',
    'Consider all code quality improvements, not only the listed findings. Do not over-fit recommendations to the quality model or assume that a FAIL status proves a defect. A passing file may also have worthwhile improvements.',
    'Improvements may extend beyond this file, including reorganizing files and folders, splitting large files along clear responsibilities, or structural changes across related files when justified.',
    'Avoid extraneous indirection and never make the code worse just to improve the analysis. Preserve intended behavior. It is valid to recommend no change.',
    'Treat this like a greenfield app where the top priorities are elegance, quality, and long-term maintainability for human software engineers.',
    '',
    'Return concrete recommendations, in priority order. For each, explain the source-level problem, the proposed refactoring, which files would change, why it helps readers, and how to verify behavior. Include useful file and symbol references. Explain which findings you agree or disagree with and why. If no refactoring is justified, say so.',
    '',
    'The data contains saved summary measurements and leading score factors, not source code or an exhaustive list of findings. Inspect the source to locate problems. Missing evidence is not a zero value. AI assessments use a 0–100 scale; their exact evaluation questions are included. Raw numeric precision is retained so small changes are not hidden by display rounding.',
    '',
    'Analysis data:',
    '```json',
    JSON.stringify(data, null, 2),
    '```',
  );
  return lines.join('\n');
}

// A prompt covering the rows shown in the week drilldown, largest declines first.
// `scope` says whether `rows` are the files the reader checked or every shown
// row, and how many rows the report showed.
function weekRefactorPrompt(rows, week, context, weekLabel, totals, scope = {kind: 'shown', shown: rows.length}) {
  const ranked = [...rows].sort((a, b) => a.contribution - b.contribution).slice(0, 25);
  const target = row => (row.after.length ? row.after : row.before)[0];
  const data = {
    repository: context.repository,
    interval: context.interval || 'week',
    period: weekLabel,
    revisions: {before: week.before_commit, after: week.commit},
    compare_url: `${context.repository}/compare/${week.before_commit}...${week.commit}`,
    period_totals: {improved: totals.positive, declined: totals.negative, net: totals.net,
      meaning: `Sum of file contributions to the project result for the selected period; the file list below covers the ${scope.kind === 'selected' ? 'files selected in the report' : 'shown rows'} only.`},
    scoring: {
      range: [1, 10],
      higher_is_better: true,
      pass_threshold: context.threshold_score,
      meaning: context.model.score_meaning,
      model_id: context.model.id,
      model_sha256: context.model.sha256,
    },
    files: ranked.map(row => ({
      path: target(row).path,
      change_type: row.kind,
      ...(row.kind === 'Renamed' ? {previous_path: row.before[0].path} : {}),
      snapshot: row.after.length ? 'after' : 'before',
      before_score: row.before_score,
      after_score: row.after_score,
      before_mass: row.before_mass,
      after_mass: row.after_mass,
      period_impact: row.contribution,
      leading_factors: (row.explanation?.factors ?? []).map(factor => ({name: factor.name, impact_in_score_points: factor.change})),
      diff_url: `${context.repository}/compare/${week.before_commit}...${week.commit}#diff-${row.diff_anchor}`,
    })),
    listed_files: ranked.length,
    shown_files: scope.shown,
    selected_files: scope.kind === 'selected' ? rows.length : null,
  };
  return [
    'This project was analyzed by qlty slop-one. Review the files that changed in one period and recommend refactorings that improve maintainability for human software engineers.',
    '',
    `Project: ${context.repository}`,
    `Analysis snapshot: ${weekLabel}`,
    '',
    'All report data for this review is included below; no separate qlty slop-one report or data files are needed. Inspect the actual source code in the project.',
    '',
    'Use the current checkout as the working context. The analysis is a historical snapshot: first check whether each file still exists and whether the findings still apply. Use the supplied revisions and diffs for context without resetting the checkout.',
    'Files are listed with the largest negative impact first. Start there, but weigh every file on its merits. Removed files describe the earlier revision; do not restore deleted code merely to address historical findings.',
    '',
    'Consider all code quality improvements, not only the listed findings. Do not over-fit recommendations to the quality model or assume that a FAIL status proves a defect. A passing file may also have worthwhile improvements.',
    'Improvements may extend beyond single files, including reorganizing files and folders, splitting large files along clear responsibilities, or structural changes across related files when justified.',
    'Avoid extraneous indirection and never make the code worse just to improve the analysis. Preserve intended behavior. It is valid to recommend no change.',
    'Treat this like a greenfield app where the top priorities are elegance, quality, and long-term maintainability for human software engineers.',
    '',
    'Return concrete recommendations, in priority order. For each, explain the source-level problem, the proposed refactoring, which files would change, why it helps readers, and how to verify behavior. Include useful file and symbol references. If no refactoring is justified for a file, say so.',
    '',
    'The data contains scores, period impacts, and leading score factors, not source code or an exhaustive list of findings. Inspect the source to locate problems. Raw numeric precision is retained so small changes are not hidden by display rounding.',
    '',
    'Analysis data:',
    '```json',
    JSON.stringify(data, null, 2),
    '```',
  ].join('\n');
}
