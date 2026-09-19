using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;
namespace UMLCAD.Framework;
public sealed class UmlcadApplication:IDisposable
{
    private readonly CadEvaluationEngine _engine;private readonly IKernelGateway _kernel;private bool _disposed;
    public UmlcadApplication(IKernelGateway kernel){_kernel=kernel??throw new ArgumentNullException(nameof(kernel));_engine=new CadEvaluationEngine(kernel);}
    public Task<CadEvaluationSnapshot> BuildAsync(CadPart part,CadEvaluationOptions? options=null,CancellationToken cancellationToken=default){Ensure();return _engine.EvaluateAsync(part,options,cancellationToken);}
    public Task<CadEvaluationSnapshot> RebuildAsync(CadPart part,IReadOnlySet<CadId> changed,CadEvaluationOptions? options=null,CancellationToken cancellationToken=default){Ensure();return _engine.RebuildAsync(part,changed,options,cancellationToken);}
    public void Dispose(){if(_disposed)return;_disposed=true;if(_kernel is IDisposable d)d.Dispose();}
    private void Ensure(){if(_disposed)throw new ObjectDisposedException(nameof(UmlcadApplication));}
}
