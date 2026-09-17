import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, appendRelations, evaluateRelation, type GeometricRelation } from "../kernel/src/native-index.js";

test("circle tangency supports explicit external and internal modes",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"circle",center:{x:0,y:0},radius:5}));
  p.addGeometry("b",()=>({kind:"circle",center:{x:2,y:0},radius:3}));
  const external:GeometricRelation={kind:"tangent",firstGeometryId:"a",secondGeometryId:"b",mode:"external"};
  const internal:GeometricRelation={kind:"tangent",firstGeometryId:"a",secondGeometryId:"b",mode:"internal"};
  assert.ok(Math.abs(evaluateRelation(p.snapshot(),external).scaledNorm)>0);
  assert.equal(evaluateRelation(p.snapshot(),internal).scaledNorm,0);
});

test("relation snapshot preserves explicit geometry-to-relation dependencies",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"line",start:{x:0,y:0},end:{x:1,y:0}}));
  p.addGeometry("b",()=>({kind:"line",start:{x:0,y:1},end:{x:1,y:1}}));
  const r:GeometricRelation={kind:"parallel",firstGeometryId:"a",secondGeometryId:"b"};
  const snapshot=appendRelations(p.snapshot(),[r]);
  assert.equal(snapshot.relationDependencies.length,2);
  assert.deepEqual(snapshot.relationDependencies.map(d=>d.from.id),["a","b"]);
  assert.ok(snapshot.relationDependencies.every(d=>d.to.kind==="relation"));
});
