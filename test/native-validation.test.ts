import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, validateEngineering, validateMigration, validateSemanticReferences } from "../kernel/src/native-index.js";

test("engineering evidence separates structural and acceptance facts",()=>{
  const p=new DesignProgram();
  p.addGeometry("edge",()=>({kind:"line",start:{x:0,y:0},end:{x:10,y:0}}));
  p.horizontal({kind:"geometry",entityId:"edge"});
  const evidence=validateEngineering(p.snapshot());
  assert.equal(evidence.structuralValidity,true);
  assert.equal(evidence.constraintValidity,true);
  assert.equal(evidence.topologyValidity,true);
  assert.equal(evidence.spatialValidity,true);
  assert.equal(evidence.exportValidity,true);
});

test("engineering evidence rejects an invalid structure",()=>{
  const p=new DesignProgram();
  p.addGeometry("edge",()=>({kind:"line",start:{x:0,y:0},end:{x:10,y:0}}));
  const snapshot={...p.snapshot(),geometry:[...p.snapshot().geometry,{id:"edge",geometry:{kind:"line" as const,start:{x:0,y:0},end:{x:1,y:1}},parameterDependencies:[]}]};
  const evidence=validateEngineering(snapshot);
  assert.equal(evidence.structuralValidity,false);
  assert.equal(evidence.engineeringRuleValidity,false);
});

test("semantic reference validation detects stale and migrated references",()=>{
  const p=new DesignProgram();
  p.addGeometry("edge",()=>({kind:"line",start:{x:0,y:0},end:{x:1,y:0}}));
  const snapshot=p.snapshot();
  assert.equal(validateSemanticReferences(snapshot,[{kind:"geometry",id:"edge"}]).length,0);
  assert.equal(validateSemanticReferences(snapshot,[{kind:"geometry",id:"missing"}])[0]!.code,"stale-reference");
  assert.equal(validateMigration({kind:"geometry",id:"old"},{old:"edge"},snapshot).length,0);
});
