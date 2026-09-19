using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class AuthoritativeResultIntegrationTests
{
    [Fact]
    public void Integrates_authoritative_result_without_changing_semantic_build_identity()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid");
        using var app = builder.Build();

        var result = ValidResult("result-r1", "part");
        var service = app.GetRequiredService<IAuthoritativeResultIntegrationService>();

        var integrated = service.Integrate(app.Semantic, result);

        Assert.Equal(app.Semantic.BuildIdentity, integrated.BuildIdentity);
        Assert.Single(integrated.AuthoritativeResults);
        Assert.Equal(result, integrated.AuthoritativeResults.Single());
    }

    [Fact]
    public void Derived_authoritative_results_are_not_emitted_as_kernel_semantic_input()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid");
        using var app = builder.Build();

        var service = app.GetRequiredService<IAuthoritativeResultIntegrationService>();
        var integrated = service.Integrate(app.Semantic, ValidResult("result-r1", "part"));
        var package = app.GetRequiredService<IBuildPackageService>().CreatePackage(integrated);

        Assert.Equal(integrated.BuildIdentity, package.BuildIdentity);
        Assert.Empty(package.Semantic.AuthoritativeResults);
        Assert.Equal(app.Semantic.BuildIdentity, package.BuildIdentity);
    }

    [Fact]
    public void Integrating_the_same_result_twice_is_idempotent()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid");
        using var app = builder.Build();

        var service = app.GetRequiredService<IAuthoritativeResultIntegrationService>();
        var first = service.Integrate(app.Semantic, ValidResult("result-r1", "part"));
        var second = service.Integrate(first, ValidResult("result-r1", "part"));

        Assert.Equal(first, second);
        Assert.Single(second.AuthoritativeResults);
    }

    [Fact]
    public void Conflicting_duplicate_result_identity_is_rejected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid");
        using var app = builder.Build();

        var service = app.GetRequiredService<IAuthoritativeResultIntegrationService>();
        var first = service.Integrate(app.Semantic, ValidResult("result-r1", "part"));

        var conflict = new AuthoritativeResultSemantic(
            "result-r1",
            "part",
            "operation-2",
            "contract-v2",
            "evidence-2",
            AuthoritativeResultStatus.Authoritative);

        var exception = Assert.Throws<InvalidOperationException>(() =>
            service.Integrate(first, conflict));

        Assert.Contains("RESULT_ID_CONFLICT", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void Result_for_unknown_producer_is_rejected()
    {
        using var app = Build(builder => builder.AddPart("part", "solid"));
        var service = app.GetRequiredService<IAuthoritativeResultIntegrationService>();

        var exception = Assert.Throws<InvalidOperationException>(() =>
            service.Integrate(app.Semantic, ValidResult("result-r1", "missing-part")));

        Assert.Contains("RESULT_PRODUCER_MISSING", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void Invalid_result_identity_and_evidence_are_rejected()
    {
        using var app = Build(builder => builder.AddPart("part", "solid"));
        var service = app.GetRequiredService<IAuthoritativeResultIntegrationService>();

        foreach (var result in new[]
        {
            new AuthoritativeResultSemantic("", "part", "operation", "contract", "evidence", AuthoritativeResultStatus.Authoritative),
            new AuthoritativeResultSemantic("result", "", "operation", "contract", "evidence", AuthoritativeResultStatus.Authoritative),
            new AuthoritativeResultSemantic("result", "part", "", "contract", "evidence", AuthoritativeResultStatus.Authoritative),
            new AuthoritativeResultSemantic("result", "part", "operation", "", "evidence", AuthoritativeResultStatus.Authoritative),
            new AuthoritativeResultSemantic("result", "part", "operation", "contract", "", AuthoritativeResultStatus.Authoritative)
        })
        {
            Assert.Throws<ArgumentException>(() => service.Integrate(app.Semantic, result));
        }
    }

    [Fact]
    public void Rejected_result_may_be_registered_but_never_becomes_authoritative()
    {
        using var app = Build(builder => builder.AddPart("part", "solid"));
        var service = app.GetRequiredService<IAuthoritativeResultIntegrationService>();

        var integrated = service.Integrate(
            app.Semantic,
            new AuthoritativeResultSemantic(
                "result-r1",
                "part",
                "operation",
                "contract",
                "evidence",
                AuthoritativeResultStatus.Rejected));

        Assert.Single(integrated.AuthoritativeResults);
        Assert.Equal(AuthoritativeResultStatus.Rejected, integrated.AuthoritativeResults[0].Status);
    }

    private static AuthoritativeResultSemantic ValidResult(string id, string producerId) =>
        new(id, producerId, "operation-1", "contract-v1", "evidence-1", AuthoritativeResultStatus.Authoritative);

    private static CadApplication Build(Action<CadApplicationBuilder> configure)
    {
        var builder = CadApplication.CreateBuilder();
        configure(builder);
        return builder.Build();
    }
}
