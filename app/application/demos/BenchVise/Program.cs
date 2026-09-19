using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;
using UMLCAD.Framework;
var part=CadPartProgram.Create("bench-vise","Bench Vise")
 .Sketch(new SketchOperation(new CadId("sketch"),new CadId("body"),new Sketch(new CadId("sketch"),"Profile",new SketchGeometry[]{new CircleGeometry(new CadId("circle"),0,0,20)},Array.Empty<SketchConstraint>(),Array.Empty<CadReference>())))
 .Extrude(new ExtrusionOperation(new CadId("extrude"),new CadId("body"),new CadId("sketch"),CadExpression.Constant(60),"+Z"))
 .Hole(new HoleOperation(new CadId("hole"),new CadId("body"),new CadId("extrude"),CadExpression.Constant(8),CadExpression.Constant(30)));
using var app=new UmlcadApplication(new DemoKernel());
var snapshot=await app.BuildAsync(part.Part);
Console.WriteLine($"Pipeline: {string.Join(" -> ",snapshot.Plan.OperationIds)}");
Console.WriteLine($"Final Body: {snapshot.CurrentBody(new CadId("body"))}");
sealed class DemoKernel:IKernelGateway
{
 public Task<KernelOperationResponse> EvaluateAsync(KernelOperationRequest r,CancellationToken c=default)=>Task.FromResult(new KernelOperationResponse(CadEvaluationStatus.Succeeded,new CadResultId("demo:"+r.OperationId.Value),"demo",new[]{new KernelTopologyBinding("operation",r.OperationId.Value)},Array.Empty<CadDiagnostic>()));
}
