using System.Net.Http.Json;
using UMLCAD.Cad.Contracts;
namespace UMLCAD.Kernel;
public sealed record UmlcadKernelOptions(Uri BaseAddress,string EvaluationPath="v2/cad/operation",TimeSpan? Timeout=null){public UmlcadKernelOptions():this(new Uri("http://127.0.0.1:8080/")){}}
public sealed class UmlcadKernelGateway:IKernelGateway,IDisposable
{
    private readonly HttpClient _client;private readonly bool _ownsClient;
    public UmlcadKernelGateway(UmlcadKernelOptions? options=null,HttpClient? client=null)
    {
        options??=new();if(!options.BaseAddress.IsAbsoluteUri||options.BaseAddress.Scheme is not("http" or "https"))throw new ArgumentException("Kernel URI must be HTTP(S).");
        if(string.IsNullOrWhiteSpace(options.EvaluationPath)||Uri.TryCreate(options.EvaluationPath,UriKind.Absolute,out _))throw new ArgumentException("Kernel operation path must be relative.");
        _client=client??new HttpClient();_ownsClient=client is null;_client.BaseAddress=options.BaseAddress;_client.Timeout=options.Timeout??TimeSpan.FromMinutes(2);EvaluationPath=options.EvaluationPath;
    }
    public string EvaluationPath{get;}
    public async Task<KernelOperationResponse> EvaluateAsync(KernelOperationRequest request,CancellationToken cancellationToken=default)
    {
        using var response=await _client.PostAsJsonAsync(EvaluationPath,request,cancellationToken);
        if(!response.IsSuccessStatusCode)return new KernelOperationResponse(CadEvaluationStatus.Failed,null,null,Array.Empty<KernelTopologyBinding>(),new[]{new CadDiagnostic("KERNEL_HTTP",$"Kernel returned HTTP {(int)response.StatusCode}.")});
        return await response.Content.ReadFromJsonAsync<KernelOperationResponse>(cancellationToken:cancellationToken)??new KernelOperationResponse(CadEvaluationStatus.Failed,null,null,Array.Empty<KernelTopologyBinding>(),new[]{new CadDiagnostic("KERNEL_EMPTY_RESPONSE","Kernel returned no response.")});
    }
    public void Dispose(){if(_ownsClient)_client.Dispose();}
}
