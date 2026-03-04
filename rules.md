TITAN PROTOCOL v2.0: SYSTEM ARCHITECT MANDATE
==============================================

1. ABSOLUTE IMPLEMENTATION (ANTI-STUB)
- ZERO PLACEHOLDERS: Creating stubs, TODOs, or 'logic to be implemented' is a hard failure.
- RECURSIVE DEVELOPMENT: If a feature requires a sub-system, develop that sub-system immediately. Do not stop until the entire execution chain is functional.
- NO MOCKS: Never simulate or fake API responses or hardware behavior. Build the real bridge.

2. HERMETIC PORTABILITY (THE "ZERO-ENV" RULE)
- CODEBASE IS THE WORLD: The codebase must contain everything needed to run. No "manual" outside steps.
- CONTAINERIZED INFRASTRUCTURE: All backend services (DuckDB, SpacetimeDB, Rust Ingestion) must be orchestrated via Docker/Nix to ensure identical environments.
- AUTOMATED SETUP: Provide a 'bootstrap' script that handles dependency downloads, DLL registration, and environment variables. The user should not have to manually install a single crate or package.

3. SOURCE-OF-TRUTH SOVEREIGNTY
- CODE OVER DOCS: Always read the source code files before acting. Ignore outdated READMEs or comments.
- AUDIT BEFORE ACTION: Search for existing logic before creating new modules. Update and refactor; do not duplicate.

4. HFT PERFORMANCE & DATA INTEGRITY
- ZERO-COPY HOTPATH: Use Protobuf binary end-to-end. No string conversion in the tick-stream.
- DETERMINISTIC LATENCY: No blocking I/O or unpredictable GC triggers in the ingestion thread.
- DYNAMIC DISCOVERY: No hardcoding symbols. Use metadata-driven reactivity for all 2,000+ symbols.

5. FRONTEND-FIRST VISIBILITY
- THE OBSERVER PRINCIPLE: Assume the user is an observer. Every backend state must be visible in the Svelte UI.
- DEEP OBSERVABILITY: Structured JSON logging and real-time performance metrics (ticks/sec) are mandatory.

6. COMPLETION INTEGRITY
- TASK DEFINITION: A task is DONE only when it is implemented, portable, tested, and documented.
- TEST-DRIVEN REALISM: Never assume code works. Write integration tests under /tests that run against real binary data.

---

These rules ensure the system remains production-grade, fully implemented, maintainable, configurable, transparent, scalable, and user-controlled — without shortcuts, hidden logic, technical debt masking, or silent degradation.

7. Implementation Standards

Always fully develop and integrate all stubs, placeholders, and TODOs — never delete them to silence warnings.

Replace incomplete sections with real, production-grade implementations.

Never simulate, mock, fake, or partially implement production functionality.

Temporary scaffolding must be converted into complete logic before task completion.

If a referenced module exists but is unused:

First evaluate whether it represents intended architecture.

If yes → fully implement and integrate it properly.

If no architectural value exists → follow the Removal Decision Rules (see Section 10).

Example of forbidden placeholder:

return { success: true }; // ❌ Not allowed unless derived from real logic

8. No Hardcoding & Centralized Configuration (Enhanced)

Absolutely no hardcoded:

- Business rules
- Conditional flows
- Thresholds
- API URLs
- Keys
- Feature flags
- Static responses

All dynamic values must come from:

- Central config files (/config)
- Environment variables
- Database-managed configuration
- Admin-controlled frontend panels

Configuration must be:

- Fully centralized
- Strongly typed
- Documented
- Runtime adjustable when appropriate

If a value may change in the future, it must not be hardcoded.

Duplicate configuration definitions across files are forbidden.

Use a single configuration source of truth (e.g., /src/config/system.ts).

9. DRY (Zero Duplicate Logic Policy – Strengthened)

No business logic may exist in more than one location.

If logic appears twice, it must be abstracted immediately.

Shared functionality must live in:

- /src/utils
- /src/services
- /src/core

Frontend and backend must not reimplement the same logic differently.

Shared validation logic must be centralized.

Shared schemas must be reused.

Before writing new logic, the agent must:

- Search for an existing implementation.
- Extend or reuse it if valid.
- Only create new modules if no suitable abstraction exists.

Duplication caused by "quick fixes" is strictly forbidden.

10. Logging & Debug Visibility (Deep Observability Standard)

Every critical function must contain structured logs.

Logging must include:

- File name
- Function name
- Input parameters (sanitized)
- Execution branch decisions
- Output result
- Error stack traces
- Performance timing (where relevant)

No vague logs such as:

- Error occurred
- Failed

Required structure example:

logger.error("PaymentService.processPayment failed", {
    file: "PaymentService.ts",
    function: "processPayment",
    orderId,
    executionStage: "StripeCharge",
    errorMessage: err.message,
    stack: err.stack
});

Logging must be:

- Structured (JSON-style)
- Centralized through a logging service
- Configurable via environment level

Silent failures are forbidden.

Catch blocks must either:

- Log and rethrow
- Log and return explicit error objects

Console logs in production code are not allowed — use centralized logger.

11. No Fallbacks, Simulations, or Fake Responses (Strict Mode)

Never implement:

- Fake API responses
- Silent fallback defaults
- Demo-mode responses
- Hidden retry masking
- Mock production logic

Systems must fail visibly and transparently.

If a dependency fails:

- Log detailed failure
- Surface explicit error
- Do not return synthetic "success" responses

If a feature is incomplete:

- Complete it properly
- Or block execution with explicit error

"Temporary fallback" logic is strictly prohibited.

12. Frontend-First Architecture (Expanded & Enforced)

Every backend capability must be:

- Visible
- Monitorable
- Configurable (when appropriate)
- Auditable from frontend UI

Before backend implementation, define:

- How user interacts with it
- What controls exist
- What visibility is provided
- What failure states look like

Admin controls must exist for:

- Feature flags
- System configuration
- Logs viewing (if applicable)
- System status monitoring

No hidden backend-only logic without UI visibility unless strictly infrastructure-level.

UI must expose:

- Clear system states
- Error feedback
- Loading states
- Configuration panels

Frontend, backend, and API contracts must be developed simultaneously.

Breaking changes require synchronized updates across all layers.

13. Parallel Development Discipline

Documentation must evolve with code.

API changes require:

- Swagger/OpenAPI updates
- Frontend integration update
- Wiki documentation update

No outdated documentation allowed.

Every new system requires:

- Architecture explanation
- Data flow diagram
- Configuration reference

Feature completion requires synchronized:

- Backend logic
- Frontend UI
- Tests
- Documentation

14. Testing Structure & Integrity

All tests must live under /tests.

Required structure:

- /tests/unit
- /tests/integration
- /tests/e2e

Tests must cover:

- Success paths
- Failure paths
- Edge cases

No test logic inside production files.

No fake production logic just to satisfy tests.

Tests must validate real implementations.

15. Documentation & Development Log

After every completed task:

- Update development log
- Update system wiki

Must document:

- What was implemented
- Why it was implemented
- Configuration changes
- Architectural impact
- Edge cases

Store documentation in /docs or /wiki.

No undocumented architectural decisions.

16. Import & Code Removal Decision Rules (Anti-Confusion Enforcement Rule)

The agent must never remove code blindly. Removal is allowed only under strict evaluation.

Case A: Unused Imports That Represent Intended Architecture

If unused imports are discovered:

- Investigate their architectural purpose.

If they represent planned or meaningful architecture:

- Fully implement them.
- Integrate properly.
- Ensure functionality works.
- Add tests.

Only after full implementation may they remain.

Case B: Old / Unnecessary Imports

Remove imports only if at least one condition is true:

- Developing them would cause harm or architectural degradation.
- A superior, fully implemented system already exists.
- They are obsolete and conflict with current architecture.
- It is practically impossible to implement meaningfully.
- They duplicate already centralized functionality.

Absolute Rule:

Removal is allowed only when development provides zero architectural value OR causes degradation.

The agent must prefer:
Develop → Integrate → Validate → Test
over
Delete → Silence → Ignore

17. Completion Integrity (Strict Definition of Done)

A task is NOT complete until ALL of the following are true:

- No TODOs remain.
- No placeholders remain.
- No hardcoded values remain.
- No duplicate logic exists.
- Logging is fully implemented and structured.
- Configuration is centralized.
- Frontend integration exists (if applicable).
- Tests are written and passing.
- Documentation and development log are updated.
- No unused imports remain without evaluation under Section 16.
- No simulation, fallback, or fake logic exists.

Definition of Production-Ready

The system must be:

- Fully implemented
- Fully observable
- Fully configurable
- Fully test-covered
- Fully documented
- Fully integrated across frontend and backend

If any of the above conditions fail → the task is incomplete.
