using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SystemCadVerticalSliceGateTests
{
    [Fact]
    [Trait("Gate", "RED")]
    public void Red_gate_CreateCube_face_reference_must_resolve_as_authoritative_semantic_target()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("cube", "solid", part =>
            part.Geometry("cube-body", "solid", new Dictionary<string, string>
            {
                ["primitive"] = "cube",
                ["size"] = "10"
            }));

        using var app = builder.Build();
        var result = app.GetRequiredService<ISemanticReferenceService>()
            .Resolve(app.Semantic, new SemanticReference("cube", "front-face", "face"));

        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.True(result.IsResolved);
    }

    [Fact]
    [Trait("Gate", "RED")]
    public void Red_gate_vertical_slice_must_produce_authoritative_result_and_topology_provenance()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("cube", "solid", part =>
            part.Geometry("cube-body", "solid", new Dictionary<string, string>
            {
                ["primitive"] = "cube",
                ["size"] = "10"
            }));

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();

        Assert.NotEmpty(manifest.TopologyBindings);
        Assert.NotEmpty(manifest.Representations);
    }

    [Fact]
    [Trait("Gate", "RED")]
    public void Red_gate_sketch_and_feature_must_exist_as_semantic_concepts_not_viewer_state()
    {
        var semanticTypes = typeof(CadApplication).Assembly
            .GetTypes()
            .Where(type => type.IsPublic && type.Namespace == typeof(SemanticApplication).Namespace)
            .ToArray();

        Assert.Contains(
            semanticTypes,
            type => type.Name.Contains("Sketch", StringComparison.OrdinalIgnoreCase));

        Assert.Contains(
            semanticTypes,
            type => type.Name.Contains("Feature", StringComparison.OrdinalIgnoreCase));
    }

    [Fact]
    public void Current_foundation_slice_is_provably_stopped_before_face_semantics()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("cube", "solid", part =>
            part.Geometry("cube-body", "solid", new Dictionary<string, string>
            {
                ["primitive"] = "cube",
                ["size"] = "10"
            }));

        using var app = builder.Build();
        var result = app.GetRequiredService<ISemanticReferenceService>()
            .Resolve(app.Semantic, new SemanticReference("cube", "front-face", "face"));

        Assert.Equal(SemanticReferenceStatus.Unsupported, result.Status);
        Assert.Equal("REFERENCE_UNSUPPORTED_TARGET_KIND", result.DiagnosticCode);
        Assert.Empty(app.CreateCompiledModelManifest().TopologyBindings);
    }
}
