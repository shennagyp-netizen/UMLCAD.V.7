import { validateSnapshot, type DesignProgramSnapshot, type KernelDiagnostic } from "./core.js";
import { analyzeConstraints, solveConstraints, type ConstraintAnalysis, type ConstraintSolveResult, type SolverOptions } from "./solver.js";
import { buildTopology, type TopologyModel } from "./topology.js";
import { spatialAnalysis, type SpatialResult } from "./spatial.js";
import { exportDxf } from "./dxf.js";
import { validateSemanticReferences } from "./reference-validation.js";
import { classifyConstraintSystem } from "./diagnostics.js";
import type { SemanticReference } from "./references.js";
import type { GeometricRelation } from "./relations.js";

export interface EngineeringEvidence {
  readonly structuralValidity:boolean;
  readonly constraintValidity:boolean;
  readonly relationValidity:boolean;
  readonly numericalConditioning:boolean;
  readonly referenceValidity:boolean;
  readonly topologyValidity:boolean;
  readonly spatialValidity:boolean;
  readonly engineeringRuleValidity:boolean;
  readonly exportValidity:boolean;
  readonly diagnostics:readonly KernelDiagnostic[];
  readonly constraintAnalysis:ConstraintAnalysis;
  readonly topology:TopologyModel|null;
  readonly spatial:readonly SpatialResult[];
}
export interface EngineeringSolveResult { readonly accepted:boolean; readonly solve:ConstraintSolveResult; readonly evidence:EngineeringEvidence; readonly snapshot:DesignProgramSnapshot; }

export function validateEngineering(snapshot:DesignProgramSnapshot,relations:readonly GeometricRelation[]=[],references:readonly SemanticReference[]=[]):EngineeringEvidence{
  const diagnostics:KernelDiagnostic[]=[];
  const structural=validateSnapshot(snapshot);
  diagnostics.push(...structural);
  const structuralValidity=structural.every(d=>d.severity!=="error");
  const classification=classifyConstraintSystem(snapshot,relations);
  for(const warning of classification.warnings)diagnostics.push({code:"constraint-analysis",message:warning,severity:"warning"});
  for(const id of classification.contradictoryConstraintIds)diagnostics.push({code:"contradictory-constraint",message:`Contradictory constraint: ${id}`,severity:"error"});
  for(const index of classification.contradictoryRelationIndexes)diagnostics.push({code:"contradictory-relation",message:`Contradictory relation index: ${index+1}`,severity:"error"});
  const constraintAnalysis=analyzeConstraints(snapshot,1e-8,relations);
  const constraintValidity=constraintAnalysis.valid&&constraintAnalysis.satisfied&&classification.contradictoryConstraintIds.length===0;
  const relationValidity=constraintAnalysis.valid&&constraintAnalysis.relationSatisfied&&classification.contradictoryRelationIndexes.length===0;
  // A currently satisfied rectangular/underdetermined system is not numerically rejected merely because
  // its Jacobian has unconstrained degrees of freedom. Conditioning is still required when a non-satisfied
  // system needs numerical correction; rank/DOF diagnostics remain exposed on constraintAnalysis.
  const numericalConditioning=constraintAnalysis.equationCount===0||constraintAnalysis.satisfied||constraintAnalysis.wellConditioned;
  const referenceDiagnostics=validateSemanticReferences(snapshot,references);
  diagnostics.push(...referenceDiagnostics.map(d=>({code:d.code,message:d.message,severity:d.severity} as KernelDiagnostic)));
  const referenceValidity=referenceDiagnostics.every(d=>d.severity!=="error");
  let topology:TopologyModel|null=null;
  let topologyValidity=true;
  if(structuralValidity){try{topology=buildTopology(snapshot);}catch(error){topologyValidity=false;diagnostics.push({code:"topology-invalid",message:error instanceof Error?error.message:String(error),severity:"error"});}}
  let spatial:readonly SpatialResult[]=[];
  let spatialValidity=true;
  if(structuralValidity){try{spatial=spatialAnalysis(snapshot.geometry);if(spatial.some(item=>!Number.isFinite(item.distance)))throw Error("Spatial analysis produced a non-finite distance");}catch(error){spatialValidity=false;diagnostics.push({code:"spatial-invalid",message:error instanceof Error?error.message:String(error),severity:"error"});}}
  let exportValidity=true;
  try{const dxf=exportDxf(snapshot);if(!dxf.includes("SECTION")||!dxf.includes("EOF"))throw Error("DXF export did not produce the required section terminator");}catch(error){exportValidity=false;diagnostics.push({code:"export-invalid",message:error instanceof Error?error.message:String(error),severity:"error"});}
  const engineeringRuleValidity=structuralValidity&&constraintValidity&&relationValidity&&numericalConditioning&&referenceValidity&&topologyValidity&&spatialValidity&&exportValidity;
  return Object.freeze({structuralValidity,constraintValidity,relationValidity,numericalConditioning,referenceValidity,topologyValidity,spatialValidity,engineeringRuleValidity,exportValidity,diagnostics:Object.freeze(diagnostics),constraintAnalysis,topology,spatial:Object.freeze(spatial)});
}

export function solveAndValidateEngineering(snapshot:DesignProgramSnapshot,options:SolverOptions={},relations:readonly GeometricRelation[]=[],references:readonly SemanticReference[]=[]):EngineeringSolveResult{
  const solve=solveConstraints(snapshot,{...options,relations:[...(options.relations??[]),...relations]});
  const relationSet=Object.freeze([...(options.relations??[]),...relations]);
  const geometry=Object.freeze(solve.geometry.map(item=>{const source=snapshot.geometry.find(candidate=>candidate.id===item.id)!;return Object.freeze({id:item.id,geometry:item.geometry,parameterDependencies:source.parameterDependencies});}));
  const solvedSnapshot=Object.freeze({...snapshot,geometry});
  const evidence=validateEngineering(solvedSnapshot,relationSet,references);
  const accepted=solve.converged&&evidence.engineeringRuleValidity;
  return Object.freeze({accepted,solve,evidence,snapshot:solvedSnapshot});
}
