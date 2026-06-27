---
owner: codex-bridge maintainers
status: active
updated: 2026-06-27
tags:
  - workflow
  - delivery
links:
  - progress-plan.md
  - codex-bridge-cli-design.md
---

# Development Workflow

Summary: Simplified milestone-based execution and review loop.

This project follows a "One Milestone = One Commit" workflow to ensure high code quality and a clean git history.

## 1. Milestone as a Logical Unit

All work is tracked in `docs/progress-plan.md` under specific Milestones (e.g., `M2: Real Chat Upstream Integration`).
- **Atomic Commits**: We do not commit partial work for a milestone. A milestone must be fully implemented, tested, and reviewed before it is committed.
- **Progress Tracking**: During implementation, update the item statuses (`todo` -> `doing` -> `done`) in `docs/progress-plan.md`.

## 2. Implementation & Gate (Build & Verify)

Before submitting a milestone for review, it must pass the mechanical quality gates:
1. **Complete Implementation**: All checklist tasks in the milestone are implemented.
2. **Pass Core Tests**: Run the following and ensure they are green:
   ```bash
   cargo fmt
   cargo check
   cargo test
   ```
3. **Acceptance Criteria**: Ensure the specific acceptance criteria defined for the milestone in `progress-plan.md` are met (e.g., verifying a local CLI command works or a specific integration is functional).

## 3. Review & Fix Loop

Once the gate is passed, initiate a code review (via a review subagent or a peer). The review process is bounded and focused:

### 3.1. Review Focus
- **Correctness**: Does the code fulfill the milestone's acceptance criteria?
- **Robustness**: Are error cases properly handled (no silent panics or swallowed errors)?
- **Architecture**: Is it consistent with `docs/codex-bridge-cli-design.md`?

### 3.2. Issue Classification
Review findings must be classified to prevent endless review cycles:
- **Blocker (High/Medium)**: Bugs, missing acceptance criteria, panic risks, or architecture violations. *Must be fixed in this loop.*
- **Follow-up (Low)**: Minor refactors, naming preferences, or non-critical enhancements. *Logged for the next milestone, does not block the current commit.*

### 3.3. Fix & Regate
- Fix all Blocker issues.
- Re-run the Gate (`cargo check`, `cargo test`) after fixes.
- Reviewer checks the fixes. If clear, the loop ends.

## 4. Commit

When the review loop is closed (0 Blockers), commit the milestone:
- Command: `git add <changed paths>` then `git commit -m "Milestone <ID>: <Title>"` (e.g., `Milestone M2: Real Chat Upstream Integration`)
- Only commit once the full gate and review loop is passed successfully.
