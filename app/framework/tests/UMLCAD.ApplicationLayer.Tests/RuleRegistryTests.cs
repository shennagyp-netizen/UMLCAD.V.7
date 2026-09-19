using UMLCAD.Engineering.Runtime;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class RuleRegistryTests
{
    [Fact]
    public void Resolution_Uses_Highest_Applicable_Scope()
    {
        var registry = new EngineeringRuleRegistry();
        var context = new EngineeringRuleResolutionContext(
            CompanyId: "company-a",
            ProjectId: "project-a",
            PartId: "part-a",
            ProcessId: "process-a");

        var defaultRule = new MarkerRule("rule.one", "default");
        var companyRule = new MarkerRule("rule.one", "company");
        var projectRule = new MarkerRule("rule.one", "project");
        var partRule = new MarkerRule("rule.one", "part");
        var processRule = new MarkerRule("rule.one", "process");

        registry.Register(new(defaultRule, EngineeringRuleScope.Default, null));
        registry.Register(new(companyRule, EngineeringRuleScope.Company, "company-a"));
        registry.Register(new(projectRule, EngineeringRuleScope.Project, "project-a"));
        registry.Register(new(partRule, EngineeringRuleScope.Part, "part-a"));
        registry.Register(new(processRule, EngineeringRuleScope.Process, "process-a"));

        var resolved = registry.Resolve("rule.one", context);

        Assert.Same(processRule, resolved);
    }

    [Fact]
    public void Resolution_Falls_Back_To_Lower_Scope_When_Higher_Scope_Is_Not_Applicable()
    {
        var registry = new EngineeringRuleRegistry();
        var context = new EngineeringRuleResolutionContext(
            CompanyId: "company-a",
            ProjectId: "project-a");

        var defaultRule = new MarkerRule("rule.one", "default");
        var companyRule = new MarkerRule("rule.one", "company");
        var otherProjectRule = new MarkerRule("rule.one", "other-project");

        registry.Register(new(defaultRule, EngineeringRuleScope.Default, null));
        registry.Register(new(companyRule, EngineeringRuleScope.Company, "company-a"));
        registry.Register(new(otherProjectRule, EngineeringRuleScope.Project, "project-b"));

        Assert.Same(
            companyRule,
            registry.Resolve("rule.one", context));
    }

    [Fact]
    public void Duplicate_Rule_Registration_Is_Rejected()
    {
        var registry = new EngineeringRuleRegistry();
        var first = new MarkerRule("rule.one", "1");
        var second = new MarkerRule("rule.one", "2");

        registry.Register(
            new EngineeringRuleRegistration(
                first,
                EngineeringRuleScope.Company,
                "company-a"));

        Assert.Throws<InvalidOperationException>(
            () => registry.Register(
                new EngineeringRuleRegistration(
                    second,
                    EngineeringRuleScope.Company,
                    "company-a")));
    }

    [Fact]
    public void Snapshot_Is_Deterministically_Ordered()
    {
        var registry = new EngineeringRuleRegistry();

        registry.Register(new EngineeringRuleRegistration(
            new MarkerRule("rule.b", "2"),
            EngineeringRuleScope.Default,
            null));
        registry.Register(new MarkerRuleRegistration(
            new MarkerRule("rule.a", "1"),
            EngineeringRuleScope.Company,
            "company-a"));
        registry.Register(new MarkerRuleRegistration(
            new MarkerRule("rule.a", "0"),
            EngineeringRuleScope.Default,
            null));

        var snapshot = registry.Snapshot();

        Assert.Equal(
            [
                "rule.a:Default:<global>:0",
                "rule.a:Company:company-a:1",
                "rule.b:Default:<global>:2"
            ],
            snapshot.Select(registration =>
                $"{registration.Rule.Identity.RuleId}:{registration.Scope}:{registration.ScopeKey ?? "<global>"}:{registration.Rule.Identity.ImplementationVersion}")
        );
    }

    private sealed class MarkerRule(
        string ruleId,
        string version) : IEngineeringRule
    {
        public EngineeringRuleIdentity Identity { get; } =
            new(ruleId, version);

        public ValueTask<EngineeringRuleResult> ExecuteAsync(
            EngineeringContext context,
            IEngineeringServices services,
            CancellationToken cancellationToken = default) =>
            ValueTask.FromResult(
                new EngineeringRuleResult(
                    EngineeringRuleOutcomeKind.Pass,
                    []));
    }

}
