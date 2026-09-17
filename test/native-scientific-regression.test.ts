import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, buildTopology, evaluateDimensions, geometryEndpoint, spatialAnalysis, solveConstraints, type GeometricRelation } from "../kernel/api/src/native-index.js";

test("circle semantics never fabricate topological endpoints",()=>{
  const p=new DesignProgram();
  p.addGeometry("circle",()=>({kind:"circle",center:{x:2,y:3},radius:4}));
  const circle=p.snapshot().geometry[0]!.geometry;
  assert.throws(()=>geometryEndpoint(circle,"start"),/no topological start\/end endpoint|endpoint/i);
});

test("near-tangent line and circle are resolved by exact narrow phase",()=>{
  const line={id:"line",geometry:{kind:"line" as const,start:{x:-10,y:1+1e-9},end:{x:10,y:1+1e-9}}};
  const circle={id:"circle",geometry:{kind:"circle" as const,center:{x:0,y:0},radius:1}};
  const result=spatialAnalysis([line,circle],1e-12)[0]!;
  assert.equal(result.intersects,false);
  assert.ok(result.distance>0);
});

test("closed circular and arc dimensions remain analytic",()=>{
  const p=new DesignProgram();
  p.addGeometry("circle",()=>({kind:"circle",center:{x:0,y:0},radius:4}));
  p.addGeometry("arc",()=>({kind:"arc",center:{x:0,y:0},radius:4,startAngle:0,endAngle:Math.PI/2}));
  const dimensions=evaluateDimensions(p.snapshot(),[
    {id:"c",kind:"length",firstGeometryId:"circle"},
    {id:"a",kind:"length",firstGeometryId:"arc"}
  ]);
  assert.ok(Math.abs(dimensions[0]!.value-8*Math.PI)<1e-12);
  assert.ok(Math.abs(dimensions[1]!.value-2*Math.PI)<1e-12);
});

test("ambiguous topology is rejected at the incidence boundary",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"line",start:{x:0,y:0},end:{x:1,y:0}}));
  p.addGeometry("b",()=>({kind:"line",start:{x:0,y:0},end:{x:0,y:1}}));
  p.addGeometry("c",()=>({kind:"line",start:{x:0,y:0},end:{x:-1,y:0}}));
  assert.throws(()=>buildTopology(p.snapshot()),/Non-manifold topology/);
});

test("relation solver accepts explicit analytic constraints without mutating input",()=>{
  const p=new DesignProgram();
  p.addGeometry("a",()=>({kind:"line",start:{x:0,y:0},end:{x:7,y:1}}));
  p.addGeometry("b",()=>({kind:"line",start:{x:0,y:5},end:{x:10,y:5}}));
  p.horizontal({kind:"geometry",entityId:"b"});
  const relation:GeometricRelation={kind:"parallel",firstGeometryId:"a",secondGeometryId:"b"};
  const snapshot=p.snapshot();
  const solved=solveConstraints(snapshot,{relations:[relation],maxIterations:100});
  assert.equal(solved.converged,true);
  assert.deepEqual(snapshot.geometry.find(item=>item.id==="a")!.geometry,{kind:"line",start:{x:0,y:0},end:{x:7,y:1}});
});
