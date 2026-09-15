import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, solveAndValidateEngineering } from "../kernel/src/native-index.js";

test("solve-and-validate accepts a fully valid native design",()=>{
  const p=new DesignProgram();
  p.addGeometry("edge",()=>({kind:"line",start:{x:0,y:0},end:{x:10,y:0}}));
  p.horizontal({kind:"geometry",entityId:"edge"});
  const result=solveAndValidateEngineering(p.snapshot());
  assert.equal(result.accepted,true);
  assert.equal(result.evidence.structuralValidity,true);
  assert.equal(result.evidence.engineeringRuleValidity,true);
});

test("solve-and-validate rejects an invalid reference",()=>{
  const p=new DesignProgram();
  p.addGeometry("edge",()=>({kind:"line",start:{x:0,y:0},end:{x:10,y:0}}));
  const result=solveAndValidateEngineering(p.snapshot(),{},[],[{kind:"geometry",id:"missing"}]);
  assert.equal(result.accepted,false);
  assert.equal(result.evidence.referenceValidity,false);
});
