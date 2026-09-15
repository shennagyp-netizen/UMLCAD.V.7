import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { useMemo } from 'react';
import * as THREE from 'three';
import type { CompiledModelPackage } from './types';

type G={id:string;kind:string;properties:Record<string,string>};
type P={id:string;geometry:G[]};
type O={id:string;definitionId:string;definitionKind:string;transform:{matrix:number[]};visible:boolean;suppressed:boolean};
type A={id:string;occurrences:O[]};
type Build={buildIdentity:string;applicationId:string;semantic:{parts:P[];assemblies:A[]}};
type Pt=[number,number];

const enc=(s:string)=>s.replaceAll('%','%25').replaceAll(' ','%20').replaceAll('/','%2F').replaceAll(':','%3A');
const nid=(p:string,g:string)=>`definition:part:${enc(p)}/geometry:${enc(g)}`;
const pt=(s?:string):Pt|null=>{if(!s)return null;const a=s.split(',').map(Number);return a.length===2&&a.every(Number.isFinite)?[a[0],a[1]]:null};
const circle=(g:G)=>{if(g.kind!=='circle')return null;const c=pt(g.properties.center),r=Number(g.properties.radius);return c&&Number.isFinite(r)&&r>0?{c,r}:null};
const line=(g:G)=>{if(g.kind!=='line')return null;const a=pt(g.properties.start),b=pt(g.properties.end);return a&&b?[a,b] as [Pt,Pt]:null};
const same=(a:Pt,b:Pt)=>Math.hypot(a[0]-b[0],a[1]-b[1])<1e-6;
function loops(items:G[]){const x=items.map(g=>{const p=line(g);return p?{g,p}:null}).filter((v):v is {g:G;p:[Pt,Pt]}=>!!v), r=[] as {points:Pt[];ids:string[]}[];while(x.length){const q=x.shift()!,p=[q.p[0],q.p[1]] as Pt[],ids=[q.g.id];while(!same(p[p.length-1],p[0])){const i=x.findIndex(v=>same(v.p[0],p[p.length-1])||same(v.p[1],p[p.length-1]));if(i<0)break;const n=x.splice(i,1)[0];p.push(same(n.p[0],p[p.length-1])?n.p[1]:n.p[0]);ids.push(n.g.id)}if(p.length>3&&same(p[p.length-1],p[0])){p.pop();r.push({points:p,ids})}}return r}
function area(p:Pt[]){return p.reduce((s,a,i)=>{const b=p[(i+1)%p.length];return s+a[0]*b[1]-b[0]*a[1]},0)/2}
function inside(p:Pt,q:Pt[]){let h=false;for(let i=0,j=q.length-1;i<q.length;j=i++){const a=q[i],b=q[j];if(((a[1]>p[1])!==(b[1]>p[1]))&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])h=!h}return h}
function depth(id:string){return id==='vise_base'?18:id.includes('jaw')?55:id==='handle'?10:id==='lead_screw'?20:15}
function matrix(a:number[]){return a.length===16?new THREE.Matrix4().set(...a as any):new THREE.Matrix4()}
function BuildPart({part,matrix:m}:{part:P;matrix:THREE.Matrix4}){const f=useMemo(()=>{const ls=loops(part.geometry),cs=part.geometry.map(g=>{const c=circle(g);return c?{g,...c}:null}).filter((v):v is {g:G;c:Pt;r:number}=>!!v);const used=new Set(ls.flatMap(x=>x.ids)),d=depth(part.id);return{ls,cs,used,d}},[part]);return <group matrix={m} matrixAutoUpdate={false}>{f.ls.map(l=>{const out=area(l.points)<0?[...l.points].reverse():l.points;const holes=f.cs.filter(c=>inside(c.c,out));const s=new THREE.Shape();s.moveTo(out[0][0],out[0][1]);out.slice(1).forEach(p=>s.lineTo(p[0],p[1]));s.closePath();holes.forEach(h=>{const p=new THREE.Path();p.absellipse(h.c[0],h.c[1],h.r,h.r,0,Math.PI*2,false);s.holes.push(p)});const g=new THREE.ExtrudeGeometry(s,{depth:f.d,bevelEnabled:false});return <mesh key={l.ids[0]} geometry={g}><meshStandardMaterial color="#718091" metalness={.18} roughness={.68}/></mesh>})}{f.cs.filter(c=>!Array.from(f.used).includes(c.g.id)&&!f.ls.some(l=>inside(c.c,l.points))).map(c=>{const g=new THREE.CylinderGeometry(c.r,c.r,f.d,64);g.rotateX(Math.PI/2);g.translate(c.c[0],c.c[1],f.d/2);return <mesh key={c.g.id} geometry={g}><meshStandardMaterial color="#8795a4" metalness={.3} roughness={.62}/></mesh>})}{f.ls.flatMap(l=>l.points.length?[]:[])} </group>}
function collect(build:Build){const am=new Map(build.semantic.assemblies.map(a=>[a.id,a])),pm=new Map(build.semantic.parts.map(p=>[p.id,p])),used=new Set(build.semantic.assemblies.flatMap(a=>a.occurrences.filter(o=>o.definitionKind==='assembly').map(o=>o.definitionId))),roots=build.semantic.assemblies.filter(a=>!used.has(a.id)),out:{p:P;m:THREE.Matrix4;k:string}[]=[];const walk=(id:string,parent:THREE.Matrix4,key:string)=>{const a=am.get(id);if(!a)return;for(const o of a.occurrences){if(!o.visible||o.suppressed)continue;const w=new THREE.Matrix4().multiplyMatrices(parent,matrix(o.transform.matrix));if(o.definitionKind==='part'){const p=pm.get(o.definitionId);if(p)out.push({p,m:w,k:`${key}/${o.id}`})}else walk(o.definitionId,w,`${key}/${o.id}`)}};roots.forEach(a=>walk(a.id,new THREE.Matrix4(),a.id));return out}
export default function FeatureModelViewer({compiled,build}:{compiled:CompiledModelPackage;build:Build}){const ins=useMemo(()=>collect(build),[build]);return <Canvas camera={{position:[180,140,180],fov:45}}><color attach="background" args={['#11151c']}/><ambientLight intensity={1.1}/><directionalLight position={[100,140,160]} intensity={2}/><gridHelper args={[260,26]}/>{ins.map(i=><BuildPart key={i.k} part={i.p} matrix={i.m}/>)}<OrbitControls makeDefault/></Canvas>}
export async function loadFeatureBuild(url:string){const r=await fetch(url,{cache:'no-store'});if(!r.ok)throw new Error(`Unable to load build package: HTTP ${r.status}.`);return await r.json() as Build}
