---
status: open
cost-benefit-sweh:
  timebox:
    '@value': 2
    rationale: two localized fixes (root-commit UnbornBranch path; forward stdin for -F -) + tests
    confidence: unsure
  benefit-2w:
    '@value': 0.3
    rationale: both bugs have clean workarounds; cost is occasional confusion at fresh-repo / heredoc-message moments
    confidence: unsure
---
<anthropic-skill-ownership llm-subtask />

# `commit-files`/`commit-staged`: initial commit + `-F -` (stdin) message

**Priority:** Medium (both have clean workarounds, but each is a silent/confusing failure)
**Complexity:** Low–Medium
**Context:** Hit both while scaffolding `~/repo/github.com/bukzor/basedpyright-as-pyright`
(first commit of a fresh repo, with a multi-line message). Reproduced 2026-06-27.

## Bug 1 — cannot create the initial (root) commit

In a fresh repo with no `HEAD` yet (unborn branch), `commit-files` fails:

```
$ git -C <freshrepo> init -b main
$ git -C <freshrepo> add .
$ git -C <freshrepo> commit-files . -- -m "scaffold"
Error: reference 'refs/heads/main' not found; class=Reference (4); code=UnbornBranch (-9)
```

Likely cause: the tool resolves `HEAD`/the branch ref to compute the
scoped commit, and the libgit2 path errors on `UnbornBranch` instead of
treating "no parent" as the root-commit case.

**Expected:** make the root commit (parentless) gracefully — the index/paths
are the whole tree on a first commit; there is no prior HEAD to scope against.

**Workaround used:** plain `git commit` for commit #1 (safe: a fresh repo has
no contaminated index, which is the only thing these tools guard against).

## Bug 2 — `-F -` (read message from stdin) yields an empty message

Passing `-F -` through to `git commit` (heredoc on stdin) produces an empty
message and aborts:

```
$ git -C <repo> commit-files path -- -F - <<'MSG'
my message
MSG
Error: git commit failed: Aborting commit due to empty commit message.
```

Likely cause: the wrapper shells out to / re-invokes `git commit` without
forwarding its own stdin, so `-F -` reads an empty stream.

**Expected:** either forward stdin to the underlying commit so `-F -` works,
or reject `-F -` explicitly with a message pointing at `-F <file>` / `-m`.

**Workaround used:** write the message to a file and pass `-F <file>`.

## Success Criteria

- [ ] `commit-files`/`commit-staged` can create the root commit of a repo with
      no prior `HEAD`.
- [ ] `-F -` either works (stdin forwarded) or fails loudly with a helpful hint.
- [ ] Regression tests cover both: a fresh-repo first commit, and a
      stdin-message commit.

## Notes

Both are pre-1.0 papercuts with workarounds, but each *looks* like a different
problem than it is (an unborn-branch internal error; an "empty message" abort
that hides "stdin wasn't forwarded"), so they cost debugging time.
