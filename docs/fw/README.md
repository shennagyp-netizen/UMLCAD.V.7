# UMLCAD.V.7 — Future Work (fw)

This directory contains the future architectural modifications and milestone plan for the System-CAD layer.

The files here are planning/target documents. They do not claim that the described capabilities are already implemented on main.

## Files

- ENGINEERING_PROGRAMMABILITY.md — target architecture for programmable engineering knowledge/rules/services, including read access to spatial/physical knowledge and controlled write access to CAD semantics.
- MILESTONES.md — ordered future-work milestones and acceptance gates.

## Governing distinction

~~~
.NET System-CAD
    = engineering meaning, rules, orchestration, CAD control, manufacturing meaning

UMLCAD Kernel
    = one concrete mathematical/CAD computational API

Kernel internals
    = implementation detail (currently Rust, with internal accelerators/adapters as applicable)

Phenomena simulation
    = physical/engineering phenomena computation, e.g. thermal, structural, fluid, coupled simulation
~~~

The future work does not create a second semantic CAD kernel in the rule engine and does not require engineering code to know the kernel programming language.

## Two programmable engineering modes

~~~text
Normal Engineering Rule / Program
    runs as part of the ordinary build;
    reads engineering knowledge;
    may call simulation and reuse valid cached results;
    may issue semantic CAD changes;
    may reject the build with expressive diagnostics.

Engineering Supervision Program
    deliberately constructs and revises CAD scenarios;
    uses the same semantic CAD control API;
    calls ordinary rules and simulations;
    performs assertions/comparisons;
    keeps or discards revisions.
~~~

Both modes use the same engineering knowledge and CAD-control services. The distinction is their purpose: **rules enforce engineering validity of normal builds; supervision actively drives and verifies the engineering system.**

## Programmatic control path

~~~text
Engineer code
    -> Engineering Runtime
    -> typed Engineering Context
    -> semantic CAD command/change API
    -> transactional semantic draft
    -> dependency closure / recompute
    -> UMLCAD Kernel API
    -> authoritative result
    -> simulation / fields / regions / tolerances
    -> Engineering Runtime
    -> commit, reject, or revise
~~~

A rule never writes kernel memory or private CAD internals directly. A supervision program can perform sophisticated revisions, but those revisions still pass through the same semantic command and authoritative evaluation path.
