import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, solveConstraints, type GeometricRelation } from "../kernel/src/native-index.js";

test("native solver integrates relations into the same equation system",()=>{
  const p=new DesignProgram();
  p.addGeometry("reference",()=>({kind:"line",start:{x:0,y:0},end:{x:10,y:0}}));
  p.addGeometry("target",()=>({kind:"line",start:{x:0,y:5},end:{x:8,y:6}}));
  p.horizontal({kind:"geometry",entityId:"reference"});
  const relation:GeometricRelation={kind:"parallel",firstGeometryId:"target",secondGeometryId:"reference"};
  const solved=solveConstraints(p.snapshot(),{relations:[relation],maxIterations:100});
  assert.equal(solved.converged,true);
  assert.equal(solved.analysis.relationCount,1);
  assert.equal(solved.analysis.relationSatisfied,true);
  const target=solved.geometry.find(item=>item.id==="target")!.geometry;
  assert.equal(target.kind,"line");
  assert.ok(Math.abs(target.end.y-target.start.y)<1e-7);
});

test("native solver rejects an invalid relation domain",()=>{
  const p=new DesignProgram();
  p.addGeometry("circle",()=>({kind:"circle",center:{x:0,y:0},radius:1}));
  const relation={kind:"parallel",firstGeometryId:"circle",secondGeometryId:"circle"} as never as GeometricRelation;
  const solved=solveConstraints(p.snapshot(),{relations:[relation]});
  assert.equal(solved.converged,false);
  assert.equal(solved.reason,"invalid-domain");
  assert.equal(solved.analysis.valid,false);
});
