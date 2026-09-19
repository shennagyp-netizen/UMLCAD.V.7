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
