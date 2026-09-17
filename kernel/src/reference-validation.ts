import type { DesignProgramSnapshot } from "./core.js";
import type { ReferenceMigrationMap, SemanticReference } from "./references.js";

export interface ReferenceDiagnostic { readonly code:string;readonly message:string;readonly severity:"error"|"warning"; }

export function validateSemanticReferences(snapshot:DesignProgramSnapshot,references:readonly SemanticReference[]):readonly ReferenceDiagnostic[]{
  const geometryIds=new Set(snapshot.geometry.map(item=>item.id));
  const parameterIds=new Set(snapshot.parameters.map(item=>item.id));
  const constraintIds=new Set(snapshot.constraints.map(item=>item.id));
  const diagnostics:ReferenceDiagnostic[]=[];
  for(const reference of references){
    const exists=reference.kind==="geometry"?geometryIds.has(reference.id):reference.kind==="parameter"?parameterIds.has(reference.id):reference.kind==="constraint"?constraintIds.has(reference.id):false;
    if(!exists)diagnostics.push({code:"stale-reference",message:`Unknown ${reference.kind} reference: ${reference.id}`,severity:"error"});
  }
  return Object.freeze(diagnostics);
}

export function validateMigration(source:SemanticReference,migration:ReferenceMigrationMap,snapshot:DesignProgramSnapshot):readonly ReferenceDiagnostic[]{
  const resolved=migration[source.id];
  if(resolved===undefined)return Object.freeze([{code:"unmigrated-reference",message:`No migration mapping for reference: ${source.id}`,severity:"warning"}]);
  const target:SemanticReference={...source,id:resolved};
  return validateSemanticReferences(snapshot,[target]);
}
