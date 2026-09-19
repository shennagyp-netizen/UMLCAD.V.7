using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;
namespace UMLCAD.ApplicationLayer.Tests;
public sealed class IncrementalPipelineTests
{
 [Fact]public async Task HoleChangeUsesIncrementalPathForHoleOnly()
 {
   var p=Factory.Create();var g=new Gateway();var e=new CadEvaluationEngine(g);await e.EvaluateAsync(p);g.Requests.Clear();
   var s=await e.RebuildAsync(p,new HashSet<CadId>{new("hole")});
   Assert.True(s.Succeeded);Assert.Equal(KernelEvaluationMode.Incremental,s.Mode);Assert.Equal(new[]{"Cad.Hole"},g.Requests.Select(x=>x.OperationKind));Assert.Equal(new CadResultId("r:hole"),s.CurrentBody(new CadId("body")));
 }
 static class Factory
 {
  public static CadPart Create()=>CadPartProgram.Create("p","P")
   .Sketch(new SketchOperation(new CadId("sketch"),new CadId("body"),new Sketch(new CadId("sketch"),"Sketch",new SketchGeometry[]{new CircleGeometry(new CadId("c"),0,0,10)},Array.Empty<SketchConstraint>(),Array.Empty<CadReference>())))
   .Extrude(new ExtrusionOperation(new CadId("extrude"),new CadId("body"),new CadId("sketch"),CadExpression.Constant(20),"+Z"))
   .Hole(new HoleOperation(new CadId("hole"),new CadId("body"),new CadId("extrude"),CadExpression.Constant(5),CadExpression.Constant(10))).Part;
 }
 sealed class Gateway:IKernelGateway
 {
  public List<KernelOperationRequest> Requests{get;}=[];
  public Task<KernelOperationResponse> EvaluateAsync(KernelOperationRequest r,CancellationToken c=default){Requests.Add(r);var id=r.OperationKind switch{"Cad.Sketch"=>"r:sketch","Cad.Extrusion"=>"r:extrude","Cad.Hole"=>"r:hole",_=>"r:unknown"};return Task.FromResult(KernelOperationResponse.Success(r,new CadResultId(id),"evidence",new KernelHistoryIdentity("test-history:"+r.OperationId.Value),new[]{new KernelTopologyBinding("operation",r.OperationId.Value)}));}
 }
}
