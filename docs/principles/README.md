# Shared Engineering Principles

These documents contain repository-neutral decision guidance. They are
maintained from experience across consuming repositories, initially Canic and
IcyDB.

A principle becomes binding only when a consumer adopts a reviewed Shared
Tooling revision. The consumer's `AGENTS.md` and local governance remain the
authority for product architecture, exact commands, release behavior,
deployment, and exceptions.

Current principles:

- [Simplicity and maintainability](simplicity-and-maintainability.md)
- [Decision artifact discipline](decision-artifact-discipline.md)
- [Reviewable changes](reviewable-changes.md)
- [Rust code hygiene baseline](rust-code-hygiene.md)

When experience differs between repositories, update the common decision test
here and preserve the valid local choices in consumer overlays.

