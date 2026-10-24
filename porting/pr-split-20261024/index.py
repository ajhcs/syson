#!/usr/bin/env python3
"""Render the published PR index and proposed future dependency waves."""
import json
import pathlib
OUT=pathlib.Path(__file__).resolve().parent

def main():
    split=json.loads((OUT/'existing-split.json').read_text())
    published=json.loads((OUT/'published-prs.json').read_text())
    pubs={r['id']:r for r in published['pull_requests']}
    future=json.loads((OUT/'future-prs.json').read_text())
    units=future['units']
    by_id={r['id']:r for r in units}
    depths={}
    def depth(sid):
        if sid not in depths:
            depths[sid]=1+max((depth(d) for d in by_id[sid]['dependencies']),default=-1)
        return depths[sid]
    for sid in by_id:
        depth(sid)
    lines=['# SysON PR split and parallel work handoffs','',
      'The original implementation is split into **20 open draft PRs**, with **47 planned follow-on PR briefs**. This index changes no implementation or parity status. The future units are proposals, not open implementation PRs, assignments, compiler allocations or leases.','',
      'Frozen rollup: [PR #1](https://github.com/ajhcs/syson/pull/1), source `'+split['source_revision']+'`, upstream `'+split['upstream_baseline']+'`. The rollup and its original commits remain intact. Existing dirty/untracked work, branch heads and historical evidence are not swept into this split.','',
      '## Existing implementation PRs','',
      'Review these components independently. Some PRs contain large immutable source inventories, field tables or original receipts; their size is not a feature-completion measure. The 7,728-line raw factory candidate is deliberately isolated in S08 and stays default off, with independent acceptance pending. It is a review-size exception, not the template for future work.','',
      '| Unit | Draft PR | Prerequisites | Changed files | Source/script lines |',
      '| --- | --- | --- | ---: | ---: |']
    for r in split['units']:
        deps=', '.join(r['dependencies']) or 'None'
        lines.append(f"| {r['id']} | [{r['title']}]({pubs[r['id']]['url']}) | {deps} | {r['changed_files']} | {r['source_lines']} |")
    lines += ['', 'Source/script lines include tests and declarative Rust tables; they are neither handwritten-production counts nor review estimates. Shared files change only in explicit stacks: S07 → S08 for the model, and S13 → S14 for the query adapter. Both start from coherent historical source snapshots, preserving exact bytes.','',
      '## Merge and branch protocol','',
      '1. Keep PR #1 as the preserved rollup/evidence record. Review and eventually merge the component PRs, not both the rollup and its duplicate component changes. No merge is performed or authorized by this preparation.',
      '2. S01 and S02 target main. The code foundation PRs share S02 for notices/configuration and can be reviewed in parallel. Merge prerequisites before dependents.',
      '3. A PR with one prerequisite targets that prerequisite branch. After the prerequisite lands, retarget the dependent PR to main and inspect its new diff.',
      '4. A PR with multiple prerequisites targets `codex/syson-base-<unit>-20261024`. This is an immutable dependency snapshot, not a merge destination. Once all prerequisites land, retarget that PR to main. Keep these PRs draft until this check is complete.',
      '5. Before retargeting or implementation, inspect the actual base/head and active owners again. Do not force-push existing writer branches, infer lease release, activate old frozen executors or treat inactivity as permission.',
      '6. Reconcile each branch against the current merged base and run the checks appropriate to its final diff under an allocated resource slot. Historical passing receipts remain attributed to their original executed revisions; they do not qualify new split SHAs.',
      '', 'Temporary bases contain the exact prerequisite blobs with a single synthetic snapshot parent. They do not integrate into or rewrite rust/integration. [published-prs.json](published-prs.json) records every head, base and tree identity.','',
      '## Parallel follow-on work','',
      'Each [brief](briefs/) specifies one behavior family, acceptance cases, excluded scope, a proposed path boundary and dependencies. Aim for at most 400 changed handwritten production lines and 900 authored lines including tests. Split a unit further if source closure exceeds that budget. Keep generated declarations, implementation logic and runtime captures distinguishable during review.','',
      'Recommended independent first tranche after source/lease checks: **F01** scoped GeneralView binding, **F08** reflection tables, **F10** primitive conversions, **F15** node measurement, **F20** API read contracts, **F25** document reconciliation, **F36** inspector accessibility, **F39** reference preflight and **F43** distribution notices. The source base already contains their foundation code; waiting for all split PRs to merge is not a technical requirement.','',
      '**F06 is held by the existing factory validator.** It is not available for a replacement writer. Other dependency-free units may still need source-contract preparation, admitted dependencies, exact execution review or an explicit lease. Existing resource allocation remains authoritative.','',
      'Implementation can run on disjoint modules/crates. The integrator alone wires shared lib.rs exports, Cargo manifests/locks, browser entry points and the execution backlog. A worker proposes a frozen API and requests that wiring; it does not edit those shared files concurrently. Branch independence does not grant compiler/display capacity.','']
    for wave in sorted(set(depths.values())):
        lines += [f'### Dependency wave {wave}', '', '| Planned PR | Proposed lane | Future prerequisites |', '| --- | --- | --- |']
        for r in units:
            if depths[r['id']]==wave:
                lines.append(f"| [{r['id']}: {r['title']}](briefs/{r['id']}.md) | {r['lane']} | {', '.join(r['dependencies']) or 'None; gates still apply'} |")
        lines.append('')
    lines += ['## Scope remaining after these units','',
      'The 18 permanent area anchors remain unchanged and incomplete. These 47 units initiate or advance each area; they are not an assertion that all remaining features fit in 47 PRs. Full constructor families, the 70 operations, all grammar/AST obligations, every diagram/renderer, authenticated API surface, database migrations, complete interchange, collaboration, accessibility and installed-user acceptance require additional source-enumerated sibling PRs.','',
      'After each finite family, preserve unsupported dispositions and add the next source-backed family. Do not silently expand a PR into a whole parser, backend or graphical-editor rewrite. The authoritative execution-backlog.json and held leases remain untouched by this proposed handoff index.','',
      '## Verification','',
      '- All 683 changed files at the frozen source are reconstructed exactly, including modes and blob identities.',
      '- All 20 published split tree hashes were independently recomputed from Git tree encoding.',
      '- 102 Rust path dependency references and 68 literal included-file references exist in their respective split dependency closures.',
      '- All 20 remote PRs were read back as open drafts with the expected head and base.',
      '- The 47-unit future graph is acyclic, traces all 18 area anchors and has no proposed write-scope overlap between unordered units.',
      '- No Rust, browser or upstream runtime test was rerun; no existing receipt is requalified.',
      '', '[Structural result](verification.json) · [Remote PR verification](remote-verification.json) · [Exact split manifest](existing-split.json) · [Future units](future-prs.json)', '',
      'Reproduce the local structural checks from a checkout containing the pinned Git objects:', '',
      '```sh', 'python3 -B porting/pr-split-20261024/prepare.py', 'python3 -B porting/pr-split-20261024/future.py', 'python3 -B porting/pr-split-20261024/verify.py', 'python3 -B porting/pr-split-20261024/index.py', '```', '',
      'The scripts read the immutable pinned Git objects independently of the current HEAD. All outputs stay on the verified workspace; these scripts do not compile, launch services, alter Git refs/index or update leases.','']
    (OUT/'README.md').write_text('\n'.join(lines))
    print(json.dumps(dict(split_prs=len(split['units']),future_briefs=len(units),dependency_waves=len(set(depths.values())))))

if __name__=='__main__':
    main()
