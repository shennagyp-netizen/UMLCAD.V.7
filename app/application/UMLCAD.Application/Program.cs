using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;
using UMLCAD.Framework;
var part=CadPartProgram.Create("bench-vise","Bench Vise")
 .Sketch(new SketchOperation(new CadId("sketch"),new CadId("body"),new Sketch(new CadId("sketch"),"Profile",new SketchGeometry[]{new CircleGeometry(new CadId("circle"),0,0,25)},Array.Empty<SketchConstraint>(),Array.Empty<CadReference>())))
 .Extrude(new ExtrusionOperation(new CadId("extrude"),new CadId("body"),new CadId("sketch"),CadExpression.Constant(80),"+Z"))
 .Hole(new HoleOperation(new CadId("hole"),new CadId("body"),new CadId("extrude"),CadExpression.Constant(10),CadExpression.Constant(25)));
using var app=new UmlcadApplication(new DemoKernel());
var snapshot=await app.BuildAsync(part.Part);
Console.WriteLine($"Pipeline: {string.Join(" -> ",snapshot.Plan.OperationIds)}");
Console.WriteLine($"Current Body: {snapshot.CurrentBody(new CadId("body"))}");
sealed class DemoKernel:IKernelGateway
{
 public Task<KernelOperationResponse> EvaluateAsync(KernelOperationRequest r,CancellationToken c=default)
 {
   c.ThrowIfCancellationRequested();
   var id=r.OperationKind switch{"Cad.Sketch"=>"demo:sketch","Cad.Extrusion"=>"demo:body1","Cad.Hole"=>"demo:body2",_=>"demo:unknown"};
   return Task.FromResult(new KernelOperationResponse(CadEvaluationStatus.Succeeded,new CadResultId(id),"demo:"+r.EvaluationIdentity.Value,new[]{new KernelTopologyBinding("operation",r.OperationId.Value)},Array.Empty<CadDiagnostic>()));
 }
}
