import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, classifyConstraintSystem } from "../kernel/api/src/native-index.js";

test("diagnostics detect duplicate constraints and contradictions",()=>{
  const p=new DesignProgram();
  p.addGeometry("edge",()=>({kind:"line",start:{x:0,y:0},end:{x:5,y:1}}));
  p.horizontal({kind:"geometry",entityId:"edge"});
  const snapshot={...p.snapshot(),constraints:[...p.snapshot().constraints,{id:"duplicate",constraint:{kind:"horizontal" as const,entityId:"edge"}},{id:"vertical",constraint:{kind:"vertical" as const,entityId:"edge"}}]};
  const report=classifyConstraintSystem(snapshot);
  assert.deepEqual(report.redundantConstraintIds,["duplicate"]);
  assert.ok(report.contradictoryConstraintIds.includes("vertical"));
  assert.ok(report.contradictoryConstraintIds.includes("constraint-1"));
});

test("diagnostics detect contradictory relation pairs",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"line",start:{x:0,y:0},end:{x:1,y:0}}));
  p.addGeometry("b",()=>({kind:"line",start:{x:0,y:1},end:{x:1,y:1}}));
  const report=classifyConstraintSystem(p.snapshot(),[
    {kind:"parallel",firstGeometryId:"a",secondGeometryId:"b"},
    {kind:"perpendicular",firstGeometryId:"a",secondGeometryId:"b"}
  ]);
  assert.deepEqual(report.contradictoryRelationIndexes,[0,1]);
});
