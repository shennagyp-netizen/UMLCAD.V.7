using System.Net.Http.Json;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Kernel.Integration.Tests;

public sealed class AssemblyGraphE2ETests
{
    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Top_level_assembly_and_occurrences_survive_real_kernel_boundary()
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = "assembly-blackbox";
        builder.Version = "1.0.0";

        builder.AddPart("plate", "machined_part", part =>
            part.Name("Plate")
                .Geometry("edge", "line", new Dictionary<string, string>
                {
                    ["start"] = "0,0",
                    ["end"] = "100,0"
                }));

        builder.AddAssembly("sub", "Sub Assembly", assembly =>
            assembly.Part("plate-instance", "plate", "Plate Instance"));

        builder.AddAssembly("top", "Top Assembly", assembly =>
            assembly.Assembly("sub-instance", "sub", "Sub Assembly")
                .Part("direct", "plate", "Direct Plate"));

        var package = builder.Build().CreateBuildPackage();
        using var client = new HttpClient
        {
            BaseAddress = new Uri(Environment.GetEnvironmentVariable("UMLCAD_KERNEL_URL") ?? "http://127.0.0.1:8080/"),
            Timeout = TimeSpan.FromSeconds(20)
        };

        using var response = await client.PostAsJsonAsync("v1/build/evaluate", package);
        response.EnsureSuccessStatusCode();
        var result = await response.Content.ReadFromJsonAsync<KernelResponseShape>();

        Assert.NotNull(result);
        Assert.True(result!.Succeeded, string.Join("; ", result.Diagnostics.Select(x => $"{x.Code}: {x.Message}")));
        Assert.NotNull(result.CompiledModel);

        Assert.Contains("definition:assembly:top", result.CompiledModel!.Manifest.RootNodeIds);
        var top = Assert.Single(
            result.CompiledModel.Manifest.Nodes,
            x => x.Id == "definition:assembly:top");
        Assert.Equal("Top Assembly", top.Name);
        Assert.Contains(top.ChildIds, x => x == "occurrence:top/sub-instance");
        Assert.Contains(top.ChildIds, x => x == "occurrence:top/direct");

        Assert.Contains(result.CompiledModel.Manifest.Nodes, x => x.Id == "occurrence:top/sub-instance/occurrence:plate-instance");
        Assert.Contains(result.CompiledModel.Manifest.Nodes, x => x.Name == "Sub Assembly");
        Assert.Contains(result.CompiledModel.Manifest.Nodes, x => x.Name == "Plate Instance");
        Assert.Contains(result.CompiledModel.Manifest.Relationships, x => x.Id == "relationship:occurrence:top/sub-instance:instantiates");
        Assert.Contains(result.CompiledModel.Manifest.Relationships, x => x.Id == "relationship:occurrence:top/direct:instantiates");
    }

    private sealed record KernelResponseShape(
        bool Succeeded,
        CompiledModelPackage? CompiledModel,
        IReadOnlyList<CompiledDiagnostic> Diagnostics);
}
