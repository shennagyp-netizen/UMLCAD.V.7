import type { DesignProgramSnapshot } from "./core.js";
export interface SemanticManifest{readonly schema:string;readonly modelIdentity:string;readonly parameters:readonly string[];readonly geometry:readonly {readonly id:string;readonly kind:string}[];readonly constraints:readonly string[]}
export function createManifest(s:DesignProgramSnapshot,modelIdentity:string):SemanticManifest{return{schema:"uml-cad-v5.semantic-manifest/1",modelIdentity,parameters:s.parameters.map(p=>p.id),geometry:s.geometry.map(g=>({id:g.id,kind:g.geometry.kind})),constraints:s.constraints.map(c=>c.id)}}
