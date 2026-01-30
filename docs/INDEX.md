# Rustykeen Documentation Index

**Last Updated:** 2026-01-29
**Total Documents:** 81 organized across 11 categories

This master index provides navigation and cross-references for the comprehensive documentation set covering the rustykeen KenKen solver architecture, optimization work, and deployment pipelines.

---

## Core Architecture & Design

Fundamental design decisions and system architecture.

- [@./architecture.md](architecture.md) - System architecture overview and module organization
- [@./architecture_stack.md](architecture_stack.md) - Technology stack and dependency design
- [@./design.md](design.md) - Core design principles and trade-offs
- [@./design_rationale.md](design_rationale.md) - Rationale for architectural decisions
- [@./solver_architecture.md](solver_architecture.md) - Backtracking solver internals
- [@./engineering.md](engineering.md) - Engineering practices and code quality

**See also:** [Building & Features](#building-features-and-workspace), [API Stability](#api-and-versioning)

---

## KenKen Puzzle Fundamentals

Domain knowledge about KenKen puzzles, operations, and constraints.

- [@./latin_squares.md](latin_squares.md) - Latin square constraints and theory
- [@./sgt_desc_format.md](sgt_desc_format.md) - Simon Tatham puzzle description format
- [@./propagation_semantics.md](propagation_semantics.md) - Constraint propagation rules
- [@./difficulty_grading_design.md](difficulty_grading_design.md) - Difficulty classification system
- [@./difficulty.md](difficulty.md) - Deduction tier definitions and semantics

**See also:** [Solver Implementation](#solver-implementation)

---

## Solver Implementation

Solver algorithms, deduction strategies, and search techniques.

- [@./solver_architecture.md](solver_architecture.md) - Backtracking solver design
- [@./exact_cover_matrix.md](exact_cover_matrix.md) - Dancing Links (DLX) algorithm
- [@./dlx_mapping.md](dlx_mapping.md) - DLX constraint encoding
- [@./mrv_dlx.md](mrv_dlx.md) - Minimum Remaining Values heuristic
- [@./sat_cage_encoding.md](sat_cage_encoding.md) - SAT-based cage constraint encoding
- [@./cnf_templates.md](cnf_templates.md) - CNF template patterns for SAT encoding
- [@./formal_verification.md](formal_verification.md) - Z3 verification backend

**See also:** [Performance & Optimization](#performance-and-optimization)

---

## Performance & Optimization

Optimization work, profiling, and performance improvements.

### Tier 1: Cache & Domain Representations

- [@./tier1_empirical_analysis.md](tier1_empirical_analysis.md) - Tier 1.1 cage tuple caching results
- [@./tier1_optimization_analysis.md](tier1_optimization_analysis.md) - Tier 1.x analysis
- [@./tier12_domain_constraint_filtering.md](tier12_domain_constraint_filtering.md) - Tier 1.2 implementation
- [@./tier13_reevaluation_with_tier12.md](tier13_reevaluation_with_tier12.md) - Tier 1.3 cost-benefit analysis
- [@./domain_representation_analysis.md](domain_representation_analysis.md) - Domain type comparisons
- [@./domain_representation_selection.md](domain_representation_selection.md) - Selection criteria for domain types

### Tier 2: Constraint Propagation

- [@./tier21_findings.md](tier21_findings.md) - Partial constraint checking analysis
- [@./tier21_implementation_plan.md](tier21_implementation_plan.md) - Tier 2.1 design
- [@./tier21_propagation_optimization.md](tier21_propagation_optimization.md) - Propagation optimizations
- [@./tier22_final_decision.md](tier22_final_decision.md) - MRV heuristic final decision
- [@./tier22_mrv_optimization_plan.md](tier22_mrv_optimization_plan.md) - Tier 2.2 MRV design
- [@./tier22_post_implementation_analysis.md](tier22_post_implementation_analysis.md) - MRV results
- [@./tier23_lcv_evaluation.md](tier23_lcv_evaluation.md) - Least Constraining Value evaluation
- [@./tier23_lcv_measurement.md](tier23_lcv_measurement.md) - LCV benchmarking
- [@./tier23_implementation_results.md](tier23_implementation_results.md) - LCV implementation results

### Benchmarking & Profiling

- [@./benchmark_baselines.md](benchmark_baselines.md) - Performance baseline methodology
- [@./profiling_analysis.md](profiling_analysis.md) - Profiling methodology and tools
- [@./profiling_insights_tier12_deployed.md](profiling_insights_tier12_deployed.md) - Tier 1.2 profiling insights
- [@./multidimensional_profiling_tier12_analysis.md](multidimensional_profiling_tier12_analysis.md) - Comprehensive Tier 1.2 analysis
- [@./PHASE5_PGO_ANALYSIS.md](PHASE5_PGO_ANALYSIS.md) - Profile-Guided Optimization results
- [@./OPTIMIZATION_SUMMARY_2026.md](OPTIMIZATION_SUMMARY_2026.md) - 2026 optimization summary

### Optimization Roadmaps

- [@./optimization_roadmap.md](optimization_roadmap.md) - Multi-tier optimization strategy
- [@./OPTIMIZATION_ROADMAP.md](OPTIMIZATION_ROADMAP.md) - Comprehensive optimization roadmap
- [@./OPTIMIZATION_ROADMAP_FINAL.md](OPTIMIZATION_ROADMAP_FINAL.md) - Final optimization roadmap
- [@./optimization_session_tier1.md](optimization_session_tier1.md) - Tier 1 implementation guide

**See also:** [Telemetry & Instrumentation](#telemetry-and-instrumentation)

---

## Building, Features, and Workspace

Build system, feature flags, and workspace organization.

- [@./riced_build.md](riced_build.md) - Release build configuration (LTO, strip)
- [@./rust_build_system.md](rust_build_system.md) - Rust build system and targets
- [@./feature_gating.md](feature_gating.md) - Feature flag organization
- [@./features.md](features.md) - Feature descriptions and dependencies
- [@./crate_feature_plan.md](crate_feature_plan.md) - Per-crate feature strategy
- [@./crates_audit.md](crates_audit.md) - Crate organization audit
- [@./cargo_mobile2.md](cargo_mobile2.md) - cargo-mobile build integration

**See also:** [Dependencies](#dependencies)

---

## Dependencies

Dependency audits and management strategies.

- [@./dependencies.md](dependencies.md) - Workspace dependency listing
- [@./dependency_audit.md](dependency_audit.md) - External dependency analysis
- [@./dependency_matrix.md](dependency_matrix.md) - Dependency compatibility matrix
- [@./vendor_neutral.md](vendor_neutral.md) - Vendor-neutral licensing approach

**See also:** [Building, Features, and Workspace](#building-features-and-workspace)

---

## Mobile & Platform Deployment

Android/iOS deployment and cross-platform support.

### Android

- [@./android_build.md](android_build.md) - Android NDK build setup
- [@./android_deployment.md](android_deployment.md) - Android app deployment guide
- [@./android_rust_state.md](android_rust_state.md) - Current Android integration status

### Phoronix Test Suite

- [@./pts_integration_research.md](pts_integration_research.md) - PTS integration exploration
- [@./PTS_INTEGRATION_SUMMARY.md](PTS_INTEGRATION_SUMMARY.md) - PTS integration completion
- [@./pts_practical_guide.md](pts_practical_guide.md) - PTS practical usage guide
- [@./pts_quick_reference.md](pts_quick_reference.md) - PTS command reference

**See also:** [Telemetry & Instrumentation](#telemetry-and-instrumentation)

---

## Telemetry & Instrumentation

Profiling, tracing, and instrumentation tools.

- [@./tracing_tracy.md](tracing_tracy.md) - Tracy profiler integration
- [@./telemetry_build_assets.md](telemetry_build_assets.md) - Telemetry build configuration
- [@./lazy_dlx_solvercontext.md](lazy_dlx_solvercontext.md) - Solver context instrumentation

**See also:** [Performance & Optimization](#performance-and-optimization)

---

## Serialization & I/O

Data serialization and puzzle formats.

- [@./rkyv_snapshot_v2.md](rkyv_snapshot_v2.md) - Rkyv snapshot serialization format v2
- [@./uniffi_codegen.md](uniffi_codegen.md) - UniFFI foreign function interface bindings

---

## API & Versioning

Stability guarantees and version compatibility.

- [@./api_stability.md](api_stability.md) - Semantic versioning and API stability policy
- [@./CLAUDE.md](../CLAUDE.md) - Project-specific guidelines (top-level)

**See also:** [Core Architecture & Design](#core-architecture--design)

---

## Process & Planning

Planning, checklists, and methodologies.

### Cleanroom Porting

- [@./cleanroom_plan.md](cleanroom_plan.md) - Cleanroom porting strategy
- [@./cleanroom_policy.md](cleanroom_policy.md) - Cleanroom implementation policy
- [@./upstream_sgt_puzzles_keen.md](upstream_sgt_puzzles_keen.md) - Simon Tatham reference behavior

### General Process

- [@./plan.md](plan.md) - Implementation plan and roadmap
- [@./roadmap_2026.md](roadmap_2026.md) - 2026 roadmap with completion status
- [@./work_done.md](work_done.md) - Completed work tracking (2026-01-29)
- [@./dev_workflow.md](dev_workflow.md) - Development workflow and best practices
- [@./checklist.md](checklist.md) - Implementation and verification checklists

### Audits & Analysis

- [@./lacunae_audit.md](lacunae_audit.md) - Gap analysis of missing functionality
- [@./lacunae_deep_dive.md](lacunae_deep_dive.md) - Detailed lacunae investigation
- [@./target_matrix.md](target_matrix.md) - Build target support matrix
- [@./security.md](security.md) - Security considerations and constraints

---

## Corpus & Test Data

Test puzzles and benchmarking data.

- [@./corpus.md](corpus.md) - Golden test corpus documentation

---

## Navigation Guide

### By Task Type

**Getting Started:**
1. Read [@./architecture.md](architecture.md) for system overview
2. Check [@./dev_workflow.md](dev_workflow.md) for development process
3. Review [@./features.md](features.md) for available features

**Working on Solver:**
1. Start: [@./solver_architecture.md](solver_architecture.md)
2. Algorithms: [@./exact_cover_matrix.md](exact_cover_matrix.md), [@./sat_cage_encoding.md](sat_cage_encoding.md)
3. Optimization: [@./optimization_roadmap.md](optimization_roadmap.md)

**Profiling & Performance:**
1. Overview: [@./profiling_analysis.md](profiling_analysis.md)
2. Baselines: [@./benchmark_baselines.md](benchmark_baselines.md)
3. Results: [@./OPTIMIZATION_SUMMARY_2026.md](OPTIMIZATION_SUMMARY_2026.md)

**Deploying to Mobile:**
1. Android: [@./android_build.md](android_build.md) → [@./android_deployment.md](android_deployment.md)
2. General: [@./feature_gating.md](feature_gating.md), [@./crate_feature_plan.md](crate_feature_plan.md)

**Understanding Puzzle Domain:**
1. Basics: [@./latin_squares.md](latin_squares.md), [@./sgt_desc_format.md](sgt_desc_format.md)
2. Constraints: [@./propagation_semantics.md](propagation_semantics.md)
3. Difficulty: [@./difficulty_grading_design.md](difficulty_grading_design.md)

### By Architecture Layer

**Puzzle Model:**
- [@./sgt_desc_format.md](sgt_desc_format.md), [@./latin_squares.md](latin_squares.md)

**Constraint System:**
- [@./propagation_semantics.md](propagation_semantics.md), [@./solver_architecture.md](solver_architecture.md)

**Search & Deduction:**
- [@./exact_cover_matrix.md](exact_cover_matrix.md), [@./sat_cage_encoding.md](sat_cage_encoding.md)

**Optimization:**
- [@./optimization_roadmap.md](optimization_roadmap.md) and Tier docs

**Deployment:**
- [@./riced_build.md](riced_build.md), [@./android_build.md](android_build.md)

---

## Statistics

| Category | Document Count |
|----------|---|
| Architecture & Design | 6 |
| Puzzle Fundamentals | 5 |
| Solver Implementation | 7 |
| Performance & Optimization | 24 |
| Building & Workspace | 7 |
| Dependencies | 4 |
| Mobile & Deployment | 7 |
| Telemetry & Instrumentation | 3 |
| Serialization & I/O | 2 |
| API & Versioning | 2 |
| Process & Planning | 10 |
| Corpus & Test Data | 1 |
| **Total** | **81** |

---

## Contributing

For contribution guidelines, see [CONTRIBUTING.md](../CONTRIBUTING.md).

For version stability guarantees, see [@./api_stability.md](api_stability.md).

For project-level rules and practices, see [CLAUDE.md](../CLAUDE.md).

---

**Quick Links:**
- [Main README](../README.md)
- [Architectural Overview](architecture.md)
- [Optimization Roadmap](optimization_roadmap.md)
- [API Stability](api_stability.md)
- [Work Done (2026-01-29)](work_done.md)
