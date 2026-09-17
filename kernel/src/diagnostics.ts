import { evaluateRelation, type GeometricRelation } from "./relations.js";
import type { Constraint, DesignProgramSnapshot } from "./core.js";

export interface DiagnosticClassification {
  readonly redundantConstraintIds:readonly string[];
  readonly contradictoryConstraintIds:readonly string[];
  readonly contradictoryRelationIndexes:readonly number[];
  readonly warnings:readonly string[];
}

function canonical(value:unknown):string{if(value===null||typeof value!=="object")return JSON.stringify(value);if(Array.isArray(value))return`[${value.map(canonical).join(",")}]`;const record=value as Record<string,unknown>;return`{${Object.keys(record).sort().map(key=>`${JSON.stringify(key)}:${canonical(record[key])}`).join(",")}}`;}
function isConstraint(value:unknown):value is Constraint{return!!value&&typeof value==="object"&&typeof (value as {readonly kind?:unknown}).kind==="string";}

export function classifyConstraintSystem(snapshot:DesignProgramSnapshot,relations:readonly GeometricRelation[]=[]):DiagnosticClassification{
  const seen=new Map<string,string>(),redundant:string[]=[],contradictory:string[]=[],warnings:string[]=[];
  for(const item of snapshot.constraints){const key=canonical(item.constraint);const previous=seen.get(key);if(previous){redundant.push(item.id);warnings.push(`Constraint ${item.id} duplicates ${previous}`);}else seen.set(key,item.id);}
  const byEntity=new Map<string,Constraint[]>();
  for(const item of snapshot.constraints){if(!isConstraint(item.constraint))continue;const c=item.constraint;const id=c.kind==="horizontal"||c.kind==="vertical"||c.kind==="fixed"?c.entityId:c.firstGeometryId;const list=byEntity.get(id)??[];list.push(c);byEntity.set(id,list);}
  for(const [id,list] of byEntity){const hasH=list.some(c=>c.kind==="horizontal"),hasV=list.some(c=>c.kind==="vertical");if(hasH&&hasV){for(const item of snapshot.constraints)if((item.constraint.kind==="horizontal"||item.constraint.kind==="vertical")&&item.constraint.entityId===id)contradictory.push(item.id);warnings.push(`Geometry ${id} cannot satisfy both horizontal and vertical constraints`);}
    const distances=list.filter((c):c is Extract<Constraint,{kind:"distance"}>=>c.kind==="distance"&&!c.secondGeometryId&&c.firstGeometryId===id);const values=new Set(distances.map(c=>c.value));if(values.size>1){for(const item of snapshot.constraints)if(item.constraint.kind==="distance"&&item.constraint.firstGeometryId===id&&!item.constraint.secondGeometryId)contradictory.push(item.id);warnings.push(`Geometry ${id} has incompatible scalar distance constraints`);}}
  const contradictoryRelationIndexes:number[]=[];
  for(let i=0;i<relations.length;i+=1){for(let j=i+1;j<relations.length;j+=1){const a=relations[i]!,b=relations[j]!;if(a.kind==="parallel"&&b.kind==="perpendicular"&&sameUnorderedPair(a.firstGeometryId,a.secondGeometryId,b.firstGeometryId,b.secondGeometryId))contradictoryRelationIndexes.push(i,j);if(a.kind==="perpendicular"&&b.kind==="parallel"&&sameUnorderedPair(a.firstGeometryId,a.secondGeometryId,b.firstGeometryId,b.secondGeometryId))contradictoryRelationIndexes.push(i,j);if(a.kind==="radius"&&b.kind==="radius"&&a.geometryId===b.geometryId&&a.value!==b.value)contradictoryRelationIndexes.push(i,j);if(a.kind==="diameter"&&b.kind==="diameter"&&a.geometryId===b.geometryId&&a.value!==b.value)contradictoryRelationIndexes.push(i,j);}}
  return Object.freeze({redundantConstraintIds:Object.freeze([...new Set(redundant)]),contradictoryConstraintIds:Object.freeze([...new Set(contradictory)]),contradictoryRelationIndexes:Object.freeze([...new Set(contradictoryRelationIndexes)].sort((a,b)=>a-b)),warnings:Object.freeze(warnings)});
}

function sameUnorderedPair(a:string,b:string,c:string,d:string):boolean{return(a===c&&b===d)||(a===d&&b===c);}

export function relationConsistency(snapshot:DesignProgramSnapshot,relations:readonly GeometricRelation[]):readonly string[]{const out:string[]=[];for(let i=0;i<relations.length;i+=1){try{const r=evaluateRelation(snapshot,relations[i]!);if(!Number.isFinite(r.scaledNorm))out.push(`Relation ${i+1} produced a non-finite residual`);}catch(error){out.push(`Relation ${i+1} invalid: ${error instanceof Error?error.message:String(error)}`);}}return Object.freeze(out);}
