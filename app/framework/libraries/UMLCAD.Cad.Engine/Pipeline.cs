using System.Collections.ObjectModel;
using System.Security.Cryptography;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;
namespace UMLCAD.Cad.Engine;
public sealed record CadEvaluationPlan(IReadOnlyList<CadId> OperationIds);
public sealed class CadDependencyGraph
{
    private readonly IReadOnlyDictionary<CadId,IReadOnlyList<CadId>> _deps;private readonly IReadOnlyDictionary<CadId,IReadOnlyList<CadId>> _dependents;
    public CadDependencyGraph(CadPart part)
    {
        var ops=part.Operations.ToDictionary(x=>x.Id);var deps=new Dictionary<CadId,IReadOnlyList<CadId>>();
        foreach(var op in ops.Values){var inputs=op.InputOperationIds.Distinct().OrderBy(x=>x.Value,StringComparer.Ordinal).ToArray();foreach(var input in inputs)if(!ops.ContainsKey(input))throw new InvalidOperationException($"Operation '{op.Id.Value}' consumes missing operation '{input.Value}'.");deps[op.Id]=inputs;}
        var rev=ops.Keys.ToDictionary(x=>x,_=>new List<CadId>());foreach(var p in deps)foreach(var d in p.Value)rev[d].Add(p.Key);
        _deps=deps;_dependents=rev.ToDictionary(x=>x.Key,x=>(IReadOnlyList<CadId>)x.Value.OrderBy(y=>y.Value,StringComparer.Ordinal).ToArray());
    }
    public IReadOnlyList<CadId> DependenciesOf(CadId id)=>_deps.TryGetValue(id,out var d)?d:Array.Empty<CadId>();
    public IReadOnlySet<CadId> InvalidationClosure(IEnumerable<CadId> changed)
    {
        var result=new HashSet<CadId>();var q=new Queue<CadId>(changed.Distinct().OrderBy(x=>x.Value,StringComparer.Ordinal));
        while(q.Count>0){var id=q.Dequeue();if(!result.Add(id))continue;if(_dependents.TryGetValue(id,out var next))foreach(var n in next)q.Enqueue(n);}return result;
    }
    public CadEvaluationPlan Plan()
    {
        var indegree=_deps.ToDictionary(x=>x.Key,x=>x.Value.Count);var outEdges=_deps.Keys.ToDictionary(x=>x,_=>new List<CadId>());
        foreach(var p in _deps)foreach(var d in p.Value)outEdges[d].Add(p.Key);
        var ready=new SortedSet<CadId>(Comparer<CadId>.Create((a,b)=>StringComparer.Ordinal.Compare(a.Value,b.Value)));foreach(var p in indegree.Where(x=>x.Value==0))ready.Add(p.Key);
        var order=new List<CadId>();while(ready.Count>0){var c=ready.Min;ready.Remove(c);order.Add(c);foreach(var d in outEdges[c].OrderBy(x=>x.Value,StringComparer.Ordinal))if(--indegree[d]==0)ready.Add(d);}
        if(order.Count!=_deps.Count)throw new InvalidOperationException("CAD semantic operation graph contains a cycle.");
        return new CadEvaluationPlan(new ReadOnlyCollection<CadId>(order));
    }
}
public sealed record CadEvaluationOptions(string ConfigurationIdentity="default",string TolerancePolicyIdentity="default",string RepresentationPolicyIdentity="default");
public sealed record CadEvaluationOutcome(CadId OperationId,string OperationKind,CadEvaluationIdentity EvaluationIdentity,CadEvaluationStatus Status,CadResult? Result,IReadOnlyList<CadDiagnostic> Diagnostics);
public sealed record CadEvaluationSnapshot(CadPart Part,CadEvaluationPlan Plan,KernelEvaluationMode Mode,IReadOnlySet<CadId> InvalidatedOperationIds,IReadOnlyDictionary<CadId,CadEvaluationOutcome> Outcomes,IReadOnlyDictionary<CadId,CadResultId?> CurrentBodyResults)
{
    public bool Succeeded=>Outcomes.Count==Part.Operations.Count&&Outcomes.Values.All(x=>x.Status==CadEvaluationStatus.Succeeded);
    public CadResultId? CurrentBody(CadId bodyId)=>CurrentBodyResults.TryGetValue(bodyId,out var id)?id:null;
}
public interface ICadOperationCache{bool TryGet(CadEvaluationIdentity id,out CadEvaluationOutcome outcome);void Put(CadEvaluationIdentity id,CadEvaluationOutcome outcome);}
public sealed class InMemoryCadOperationCache:ICadOperationCache
{
    private readonly Dictionary<string,CadEvaluationOutcome> _cache=new(StringComparer.Ordinal);
    public bool TryGet(CadEvaluationIdentity id,out CadEvaluationOutcome outcome)=>_cache.TryGetValue(id.Value,out outcome!);
    public void Put(CadEvaluationIdentity id,CadEvaluationOutcome outcome)=>_cache[id.Value]=outcome;
}
public static class CadEvaluationIdentityBuilder
{
    public static CadEvaluationIdentity Build(string partId,CadOperation op,IReadOnlyList<CadResult> upstream,CadEvaluationOptions options)
    {
        var b=new StringBuilder(partId).Append('|').Append(op.CanonicalDefinition()).Append("|configuration=").Append(options.ConfigurationIdentity).Append("|tolerance=").Append(options.TolerancePolicyIdentity).Append("|representation=").Append(options.RepresentationPolicyIdentity);
        foreach(var r in upstream.OrderBy(x=>x.Id.Value,StringComparer.Ordinal))b.Append("|upstream=").Append(r.Id.Value).Append(':').Append(r.EvidenceHash);
        return new CadEvaluationIdentity("sha256:"+Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(b.ToString()))).ToLowerInvariant());
    }
}
public sealed class CadEvaluationEngine
{
    private readonly IKernelGateway _kernel;private readonly ICadOperationCache _cache;private readonly Dictionary<CadId,CadEvaluationOutcome> _previous=new();
    public CadEvaluationEngine(IKernelGateway kernel,ICadOperationCache? cache=null){_kernel=kernel??throw new ArgumentNullException(nameof(kernel));_cache=cache??new InMemoryCadOperationCache();}
    public Task<CadEvaluationSnapshot> EvaluateAsync(CadPart part,CadEvaluationOptions? options=null,CancellationToken cancellationToken=default)=>EvaluateCoreAsync(part,null,options,cancellationToken);
    public Task<CadEvaluationSnapshot> RebuildAsync(CadPart part,IReadOnlySet<CadId> changed,CadEvaluationOptions? options=null,CancellationToken cancellationToken=default)=>EvaluateCoreAsync(part,changed,options,cancellationToken);
    private async Task<CadEvaluationSnapshot> EvaluateCoreAsync(CadPart part,IReadOnlySet<CadId>? changed,CadEvaluationOptions? options,CancellationToken cancellationToken)
    {
        options??=new();var graph=new CadDependencyGraph(part);var plan=graph.Plan();var incremental=changed is not null;var invalidated=incremental?graph.InvalidationClosure(changed!):part.Operations.Select(x=>x.Id).ToHashSet();
        var byId=part.Operations.ToDictionary(x=>x.Id);var outcomes=new Dictionary<CadId,CadEvaluationOutcome>();var bodies=part.Bodies.ToDictionary(x=>x.Id,_=>(CadResultId?)null);
        foreach(var id in plan.OperationIds)
        {
            cancellationToken.ThrowIfCancellationRequested();var op=byId[id];var upstream=graph.DependenciesOf(id).Select(x=>outcomes[x]).ToArray();
            if(upstream.Any(x=>x.Status!=CadEvaluationStatus.Succeeded)){outcomes[id]=new CadEvaluationOutcome(id,op.OperationKind,new CadEvaluationIdentity("blocked:"+id.Value),CadEvaluationStatus.Failed,null,new[]{new CadDiagnostic("DEPENDENCY_FAILED","Unsuccessful upstream operation blocks this operation.",CadEvaluationStatus.Failed,id)});break;}
            var upstreamResults=upstream.Select(x=>x.Result!).ToArray();var identity=CadEvaluationIdentityBuilder.Build(part.Id.Value,op,upstreamResults,options);
            if(incremental&&!invalidated.Contains(id)&&_previous.TryGetValue(id,out var previous)&&previous.EvaluationIdentity==identity&&_cache.TryGet(identity,out var cached)){outcomes[id]=cached;if(cached.Result?.Kind==CadResultKind.Body)bodies[op.BodyId]=cached.Result.Id;continue;}
            var inputResults=upstreamResults.Select(x=>x.Id).ToArray();
            var request=new KernelOperationRequest(CadContractVersions.KernelOperation,identity,part.Id.Value,op.Id,op.OperationKind,incremental?KernelEvaluationMode.Incremental:KernelEvaluationMode.Full,incremental?inputResults.LastOrDefault():null,inputResults,op.SemanticInputs);
            KernelOperationResponse response;
            try{response=await _kernel.EvaluateAsync(request,cancellationToken);}catch(OperationCanceledException){throw;}catch(Exception ex){response=new KernelOperationResponse(CadEvaluationStatus.Failed,null,null,Array.Empty<KernelTopologyBinding>(),new[]{new CadDiagnostic("KERNEL_EXCEPTION",ex.Message,CadEvaluationStatus.Failed,id)});}
            if(response.Status!=CadEvaluationStatus.Succeeded||response.AuthoritativeResultId is null||string.IsNullOrWhiteSpace(response.EvidenceHash)){outcomes[id]=new CadEvaluationOutcome(id,op.OperationKind,identity,response.Status,null,response.Diagnostics);break;}
            var kind=op is SketchOperation?CadResultKind.SketchProfile:CadResultKind.Body;
            var result=new CadResult(response.AuthoritativeResultId.Value,kind,id,inputResults,response.EvidenceHash!,response.Topology);
            var outcome=new CadEvaluationOutcome(id,op.OperationKind,identity,response.Status,result,response.Diagnostics);outcomes[id]=outcome;_cache.Put(identity,outcome);if(kind==CadResultKind.Body)bodies[op.BodyId]=result.Id;
        }
        _previous.Clear();foreach(var x in outcomes)_previous[x.Key]=x.Value;
        return new CadEvaluationSnapshot(part,plan,incremental?KernelEvaluationMode.Incremental:KernelEvaluationMode.Full,invalidated,new ReadOnlyDictionary<CadId,CadEvaluationOutcome>(outcomes),new ReadOnlyDictionary<CadId,CadResultId?>(bodies));
    }
}
