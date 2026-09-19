using System.Collections.ObjectModel;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Expressions;
namespace UMLCAD.Cad.Semantics;
public enum SketchGeometryKind{Line,Circle,Arc}
public enum SketchConstraintKind{Horizontal,Vertical,Coincident,Fixed,Distance,Radius,Diameter,Tangent}
public enum ReferenceTargetKind{OperationResult,Topology,Publication,Semantic}
public abstract record SketchGeometry(CadId Id,SketchGeometryKind Kind);
public sealed record LineGeometry(CadId Id,double X1,double Y1,double X2,double Y2):SketchGeometry(Id,SketchGeometryKind.Line);
public sealed record CircleGeometry(CadId Id,double X,double Y,double Radius):SketchGeometry(Id,SketchGeometryKind.Circle)
{
    public CircleGeometry{if(!double.IsFinite(Radius)||Radius<=0)throw new ArgumentOutOfRangeException(nameof(Radius));}
}
public sealed record ArcGeometry(CadId Id,double CenterX,double CenterY,double Radius,double StartAngle,double EndAngle):SketchGeometry(Id,SketchGeometryKind.Arc);
public sealed record SketchConstraint(CadId Id,SketchConstraintKind Kind,IReadOnlyList<CadId> GeometryIds,double? Value=null);
public sealed record CadReference(CadId Id,ReferenceTargetKind TargetKind,CadId? OperationId=null,string? TopologyKind=null,string? TopologyKey=null,CadId? PublicationId=null);
public sealed record Publication(CadId Id,string Name,CadId ProducingOperationId,string TopologyKind,string TopologyKey);
public sealed record Sketch(CadId Id,string Name,IReadOnlyList<SketchGeometry> Geometry,IReadOnlyList<SketchConstraint> Constraints,IReadOnlyList<CadReference> Supports)
{
    public Sketch
    {
        if(!Id.IsValid||string.IsNullOrWhiteSpace(Name))throw new ArgumentException("Sketch identity/name is required.");
        ArgumentNullException.ThrowIfNull(Geometry);ArgumentNullException.ThrowIfNull(Constraints);ArgumentNullException.ThrowIfNull(Supports);
        if(Geometry.Select(x=>x.Id).Distinct().Count()!=Geometry.Count)throw new ArgumentException("Duplicate sketch geometry IDs.");
        if(Constraints.Select(x=>x.Id).Distinct().Count()!=Constraints.Count)throw new ArgumentException("Duplicate sketch constraint IDs.");
    }
}
public sealed record BodyDefinition(CadId Id,string Name);
public abstract record CadOperation(CadId Id,CadId BodyId,string OperationKind,IReadOnlyList<CadId> InputOperationIds)
{
    public IReadOnlyList<CadReference> References{get;init;}=Array.Empty<CadReference>();
    public IReadOnlyDictionary<string,string> SemanticInputs{get;init;}=new ReadOnlyDictionary<string,string>(new Dictionary<string,string>());
    public string CanonicalDefinition()
    {
        var b=new StringBuilder(OperationKind).Append('|').Append(Id.Value).Append('|').Append(BodyId.Value);
        foreach(var input in InputOperationIds.OrderBy(x=>x.Value,StringComparer.Ordinal))b.Append("|input-op=").Append(input.Value);
        foreach(var r in References.OrderBy(x=>x.Id.Value,StringComparer.Ordinal))b.Append("|ref=").Append(r.Id.Value).Append(':').Append(r.TargetKind).Append(':').Append(r.OperationId?.Value??"-").Append(':').Append(r.TopologyKind??"-").Append(':').Append(r.TopologyKey??"-").Append(':').Append(r.PublicationId?.Value??"-");
        foreach(var x in SemanticInputs.OrderBy(x=>x.Key,StringComparer.Ordinal))b.Append("|input=").Append(x.Key).Append('=').Append(x.Value);
        return b.ToString();
    }
}
public sealed record SketchOperation(CadId Id,CadId BodyId,Sketch Sketch):CadOperation(Id,BodyId,"Cad.Sketch",Array.Empty<CadId>);
public sealed record ExtrusionOperation(CadId Id,CadId BodyId,CadId SketchOperationId,CadExpression Distance,string Direction):CadOperation(Id,BodyId,"Cad.Extrusion",new[]{SketchOperationId})
{
    public ExtrusionOperation
    {
        ArgumentNullException.ThrowIfNull(Distance);
        if(!SketchOperationId.IsValid||string.IsNullOrWhiteSpace(Direction))throw new ArgumentException("Extrusion input is incomplete.");
        SemanticInputs=new Dictionary<string,string>{{"distance",Distance.CanonicalForm},{"direction",Direction}};
    }
}
public sealed record HoleOperation(CadId Id,CadId BodyId,CadId BaseOperationId,CadExpression Diameter,CadExpression Depth):CadOperation(Id,BodyId,"Cad.Hole",new[]{BaseOperationId})
{
    public HoleOperation
    {
        ArgumentNullException.ThrowIfNull(Diameter);ArgumentNullException.ThrowIfNull(Depth);
        if(!BaseOperationId.IsValid)throw new ArgumentException("Hole input is incomplete.");
        SemanticInputs=new Dictionary<string,string>{{"diameter",Diameter.CanonicalForm},{"depth",Depth.CanonicalForm}};
    }
}
public sealed record CadResult(CadResultId Id,CadResultKind Kind,CadId ProducingOperationId,IReadOnlyList<CadResultId> InputResults,string EvidenceHash,IReadOnlyList<KernelTopologyBinding> Topology);
public sealed record CadPart(CadId Id,string Name,IReadOnlyList<BodyDefinition> Bodies,IReadOnlyList<CadParameterBinding> Parameters,IReadOnlyList<Publication> Publications,IReadOnlyList<CadOperation> Operations)
{
    public CadPart
    {
        if(!Id.IsValid||string.IsNullOrWhiteSpace(Name))throw new ArgumentException("Part identity/name is required.");
        ArgumentNullException.ThrowIfNull(Bodies);ArgumentNullException.ThrowIfNull(Parameters);ArgumentNullException.ThrowIfNull(Publications);ArgumentNullException.ThrowIfNull(Operations);
        Unique(Bodies.Select(x=>x.Id),"body");Unique(Parameters.Select(x=>x.Name),"parameter");Unique(Publications.Select(x=>x.Id),"publication");Unique(Operations.Select(x=>x.Id),"operation");
        var bodies=Bodies.Select(x=>x.Id).ToHashSet();if(Operations.Any(x=>!bodies.Contains(x.BodyId)))throw new ArgumentException("Operation references unknown body.");
    }
    static void Unique<T>(IEnumerable<T> values,string kind){var a=values.ToArray();if(a.Distinct().Count()!=a.Length)throw new ArgumentException($"Duplicate {kind} identity.");}
}
public sealed record CadPartProgram(CadPart Part)
{
    public static CadPartProgram Create(string id,string name,string bodyId="body")=>new(new CadPart(new CadId(id),name,new[]{new BodyDefinition(new CadId(bodyId),"Main Body")},Array.Empty<CadParameterBinding>(),Array.Empty<Publication>(),Array.Empty<CadOperation>()));
    public CadPartProgram Parameter(CadParameterBinding p)=>this with{Part=Part with{Parameters=Part.Parameters.Append(p).ToArray()}};
    public CadPartProgram Publish(Publication p)=>this with{Part=Part with{Publications=Part.Publications.Append(p).ToArray()}};
    public CadPartProgram Sketch(SketchOperation op)=>AddOperation(op);
    public CadPartProgram Extrude(ExtrusionOperation op)=>AddOperation(op);
    public CadPartProgram Hole(HoleOperation op)=>AddOperation(op);
    public CadPartProgram AddOperation(CadOperation op){ArgumentNullException.ThrowIfNull(op);if(Part.Operations.Any(x=>x.Id==op.Id))throw new InvalidOperationException($"Duplicate operation '{op.Id.Value}'.");return this with{Part=Part with{Operations=Part.Operations.Append(op).ToArray()}};}
}
