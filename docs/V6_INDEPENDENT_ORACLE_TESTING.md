# UMLCAD V6 — Independent Oracle Testing

## Purpose

V6 must be tested from more than one epistemic direction. The OCCT backend is the reference geometric realization, but a test that calls OCCT and then checks the same OCCT-derived property is not an independent mathematical proof.

This document defines an additional adversarial layer: independent-oracle tests.

The oracle derives expected results from mathematics, geometry definitions, invariants, or independently implemented calculations rather than reproducing the implementation under test.

## Rule

A candidate finding from a blind external review must be reproduced against the current V6 source and checked against the V6 contract before it becomes a defect. V5 behavior is not automatically V6 semantics.

## Current coverage

The initial suite independently verifies analytic bounds for the canonical primitives, canonical box topology and Euler invariant, affine translation bounds, translation composition, and full-turn rotation identity.

Every confirmed oracle failure is promoted to a permanent regression test before the implementation is changed.
