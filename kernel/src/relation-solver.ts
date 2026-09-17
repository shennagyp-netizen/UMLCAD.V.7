import { analyzeConstraints, solveConstraints, type ConstraintAnalysis, type ConstraintSolveResult, type SolverOptions } from "./solver.js";
import type { RelationSnapshot } from "./relations.js";

export function analyzeRelations(snapshot:RelationSnapshot,tolerance=1e-8):ConstraintAnalysis{
  return analyzeConstraints(snapshot.base,tolerance,snapshot.relations.map(item=>item.relation));
}

export function solveRelations(snapshot:RelationSnapshot,options:Omit<SolverOptions,"relations">={}):ConstraintSolveResult{
  return solveConstraints(snapshot.base,{...options,relations:snapshot.relations.map(item=>item.relation)});
}
