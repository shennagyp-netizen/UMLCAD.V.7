using Xunit;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;
namespace UMLCAD.ApplicationLayer.Tests;
public sealed class SemanticPipelineTests
{
 [Fact]public async Task SketchExtrusionHoleProducesCurrentBody()
 {
   var part=CreatePart();var g=new Gateway();var s=await new CadEvaluationEngine(g).EvaluateAsync(part);
   Assert.True(s.Succeeded);Assert.Equal(new[]{new CadId("sketch"),new CadId("extrude"),new CadId("hole")},s.Plan.OperationIds);Assert.Equal(new CadResultId("r:hole"),s.CurrentBody(new CadId("body")));
 }
 [Fact]public void SketchChangeInvalidatesDownstream(){var graph=new CadDependencyGraph(CreatePart());var a=graph.InvalidationClosure(new[]{new CadId("sketch")});Assert.Equal(new[]{"extrude","hole","sketch"},a.OrderBy(x=>x.Value).Select(x=>x.Value));}
 [Fact]public void CycleFailsClosed(){var p=CadPartProgram.Create("p","P").Extrude(new ExtrusionOperation(new CadId("a"),new CadId("body"),new CadId("b"),CadExpression.Constant(1),"+Z")).Extrude(new ExtrusionOperation(new CadId("b"),new CadId("body"),new CadId("a"),CadExpression.Constant(1),"+Z"));Assert.Throws<InvalidOperationException>(()=>new CadDependencyGraph(p.Part).Plan());}
 static CadPart CreatePart()=>CadPartProgram.Create("p","P")
   .Sketch(new SketchOperation(new CadId("sketch"),new CadId("body"),new Sketch(new CadId("sketch"),"Sketch",new SketchGeometry[]{new CircleGeometry(new CadId("c"),0,0,10)},Array.Empty<SketchConstraint>(),Array.Empty<CadReference>())))
   .Extrude(new ExtrusionOperation(new CadId("extrude"),new CadId("body"),new CadId("sketch"),CadExpression.Constant(20),"+Z"))
   .Hole(new HoleOperation(new CadId("hole"),new CadId("body"),new CadId("extrude"),CadExpression.Constant(5),CadExpression.Constant(10))).Part;
 sealed class Gateway:IKernelGateway
 {
   public Task<KernelOperationResponse> EvaluateAsync(KernelOperationRequest r,CancellationToken c=default)
   {var id=r.OperationKind switch{"Cad.Sketch"=>"r:sketch","Cad.Extrusion"=>"r:extrude","Cad.Hole"=>"r:hole",_=>"r:unknown"};return Task.FromResult(KernelOperationResponse.Success(r,new CadResultId(id),"evidence",new KernelHistoryIdentity("test-history:"+r.OperationId.Value),new[]{new KernelTopologyBinding("operation",r.OperationId.Value)}));}
 }
}
