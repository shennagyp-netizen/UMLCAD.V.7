import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, appendRelations, evaluateRelation, solveRelations, validateRelation, type GeometricRelation } from "../kernel/src/native-index.js";

function base(){
  const p=new DesignProgram();
  p.addGeometry("h",()=>({kind:"line",start:{x:0,y:0},end:{x:10,y:0}}));
  p.addGeometry("v",()=>({kind:"line",start:{x:10,y:0},end:{x:10,y:10}}));
  p.addGeometry("h2",()=>({kind:"line",start:{x:0,y:5},end:{x:10,y:5}}));
  p.addGeometry("c1",()=>({kind:"circle",center:{x:0,y:0},radius:5}));
  p.addGeometry("c2",()=>({kind:"circle",center:{x:0,y:0},radius:5}));
  p.addGeometry("center",()=>({kind:"circle",center:{x:0,y:0},radius:1}));
  return p.snapshot();
}

test("parallel and perpendicular relations have zero residual on exact geometry",()=>{
  const s=base();
  const parallel:GeometricRelation={kind:"parallel",firstGeometryId:"h",secondGeometryId:"h2"};
  const perpendicular:GeometricRelation={kind:"perpendicular",firstGeometryId:"h",secondGeometryId:"v"};
  assert.equal(evaluateRelation(s,parallel).scaledNorm,0);
  assert.equal(evaluateRelation(s,perpendicular).scaledNorm,0);
});

test("equal-radius, concentric, radius and diameter relations are analytic",()=>{
  const s=base();
  assert.equal(evaluateRelation(s,{kind:"equal-radius",firstGeometryId:"c1",secondGeometryId:"c2"}).scaledNorm,0);
  assert.equal(evaluateRelation(s,{kind:"concentric",firstGeometryId:"c1",secondGeometryId:"c2"}).scaledNorm,0);
  assert.equal(evaluateRelation(s,{kind:"radius",geometryId:"c1",value:5}).scaledNorm,0);
  assert.equal(evaluateRelation(s,{kind:"diameter",geometryId:"c1",value:10}).scaledNorm,0);
});

test("point-on-line and midpoint use explicit semantic references",()=>{
  const s=base();
  assert.equal(evaluateRelation(s,{kind:"point-on-line",point:{kind:"endpoint",geometryId:"v",point:"start"},lineGeometryId:"h"}).scaledNorm,0);
  assert.ok(evaluateRelation(s,{kind:"midpoint",point:{kind:"endpoint",geometryId:"h2",point:"start"},lineGeometryId:"h"}).scaledNorm>0);
});

test("distance-points and symmetric relations expose explicit point semantics",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"line",start:{x:-5,y:0},end:{x:5,y:0}}));
  p.addGeometry("b",()=>({kind:"line",start:{x:0,y:-5},end:{x:0,y:5}}));
  p.addGeometry("center",()=>({kind:"circle",center:{x:0,y:0},radius:1}));
  const s=p.snapshot();
  assert.equal(evaluateRelation(s,{kind:"distance-points",first:{kind:"endpoint",geometryId:"a",point:"start"},second:{kind:"endpoint",geometryId:"a",point:"end"},value:10}).scaledNorm,0);
  const symmetric:GeometricRelation={kind:"symmetric",first:{kind:"endpoint",geometryId:"a",point:"start"},second:{kind:"endpoint",geometryId:"a",point:"end"},about:{kind:"center",geometryId:"center"}};
  assert.equal(evaluateRelation(s,symmetric).scaledNorm,0);
});

test("invalid relation domains fail closed",()=>{
  const s=base();
  assert.throws(()=>validateRelation(s,{kind:"parallel",firstGeometryId:"c1",secondGeometryId:"h"}),/requires a line/);
  assert.throws(()=>validateRelation(s,{kind:"radius",geometryId:"h",value:1}),/circular geometry/);
});

test("line-circle tangency is evaluated geometrically",()=>{
  const p=new DesignProgram();
  p.addGeometry("line",()=>({kind:"line",start:{x:-5,y:5},end:{x:5,y:5}}));
  p.addGeometry("circle",()=>({kind:"circle",center:{x:0,y:0},radius:5}));
  const r=evaluateRelation(p.snapshot(),{kind:"tangent",firstGeometryId:"line",secondGeometryId:"circle"});
  assert.equal(r.scaledNorm,0);
});

test("relations are stored separately from legacy constraints",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"line",start:{x:0,y:0},end:{x:10,y:0}}));
  p.addGeometry("b",()=>({kind:"line",start:{x:0,y:1},end:{x:10,y:2}}));
  const relation:GeometricRelation={kind:"parallel",firstGeometryId:"a",secondGeometryId:"b"};
  const snapshot=p.snapshot();
  const related=appendRelations(snapshot,[relation]);
  assert.equal(related.relations.length,1);
  assert.equal(snapshot.constraints.length,0);
  assert.equal(related.base.constraints.length,0);
  assert.equal(related.relationDependencies.length,2);
  assert.equal(related.base,snapshot);
  assert.notEqual(related,snapshot);
  const solved=solveRelations(related,{maxIterations:100});
  assert.equal(solved.converged,true);
  assert.ok(solved.analysis.residuals.every(residual=>residual.kind==="relation"&&residual.scaledNorm<1e-6));
});

test("relations remain part of candidate validation",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"line",start:{x:0,y:0},end:{x:1,y:0}}));
  p.addGeometry("b",()=>({kind:"line",start:{x:0,y:1},end:{x:1,y:0}}));
  const related=appendRelations(p.snapshot(),[{kind:"parallel",firstGeometryId:"a",secondGeometryId:"b"}]);
  const solved=solveRelations(related,{maxIterations:100});
  assert.equal(solved.analysis.valid,true);
  assert.equal(solved.converged,true);
  assert.equal(solved.analysis.residuals.length,1);
  assert.ok(solved.analysis.residuals[0]!.scaledNorm<1e-8);
});
