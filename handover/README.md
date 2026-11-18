# handover/ (temporary)

Everything a new build session needs to pick up where the last one stopped.
Nothing outside this folder depends on it; delete the folder (and the pointer
line in `/CLAUDE.md`) when it is no longer wanted.

- `START-HERE.md`: current state, next step, how to resume.
- `decisions.md`: Alex's rulings made during the build.
- `packages/<id>.md`: one log and evidence packet per work package.
- `design/`: copies of the design documents. Source of truth is the project's
  shared folder (`/mnt/project-files/sse4/` in a project session); re-sync with
  `handover/sync-design.sh` at every package boundary.
