namespace UMLCAD.Engineering.Runtime;

public enum EngineeringRuleScope
{
    Default = 0,
    Company = 1,
    Project = 2,
    Part = 3,
    Process = 4
}

public sealed record EngineeringRuleResolutionContext(
    string? CompanyId = null,
    string? ProjectId = null,
    string? PartId = null,
    string? ProcessId = null);

public sealed record EngineeringRuleRegistration(
    IEngineeringRule Rule,
    EngineeringRuleScope Scope,
    string? ScopeKey)
{
    public EngineeringRuleRegistration
    {
        ArgumentNullException.ThrowIfNull(Rule);

        if (Scope == EngineeringRuleScope.Default && ScopeKey is not null)
            throw new ArgumentException(
                "Default rules cannot have a scope key.",
                nameof(ScopeKey));

        if (Scope != EngineeringRuleScope.Default &&
            string.IsNullOrWhiteSpace(ScopeKey))
        {
            throw new ArgumentException(
                "Non-default rule registrations require a scope key.",
                nameof(ScopeKey));
        }
    }
}

public sealed class EngineeringRuleRegistry
{
    private readonly Dictionary<RuleRegistrationKey, EngineeringRuleRegistration> _registrations = [];

    public void Register(EngineeringRuleRegistration registration)
    {
        ArgumentNullException.ThrowIfNull(registration);

        var key = new RuleRegistrationKey(
            registration.Rule.Identity.RuleId,
            registration.Scope,
            registration.ScopeKey);

        if (!_registrations.TryAdd(key, registration))
        {
            throw new InvalidOperationException(
                $"Rule '{registration.Rule.Identity.RuleId}' is already registered for " +
                $"{registration.Scope} '{registration.ScopeKey ?? "<global>"}'.");
        }
    }

    public IEngineeringRule? Resolve(
        string ruleId,
        EngineeringRuleResolutionContext context)
    {
        if (string.IsNullOrWhiteSpace(ruleId))
            throw new ArgumentException("Rule ID cannot be empty.", nameof(ruleId));

        ArgumentNullException.ThrowIfNull(context);

        var candidates = _registrations.Values
            .Where(registration =>
                string.Equals(
                    registration.Rule.Identity.RuleId,
                    ruleId,
                    StringComparison.Ordinal))
            .Where(registration => IsApplicable(registration, context))
            .OrderByDescending(registration => registration.Scope)
            .ThenBy(
                registration => registration.ScopeKey ?? string.Empty,
                StringComparer.Ordinal)
            .ThenBy(
                registration => registration.Rule.Identity.ImplementationVersion,
                StringComparer.Ordinal)
            .ToArray();

        return candidates.FirstOrDefault()?.Rule;
    }

    public IReadOnlyList<EngineeringRuleRegistration> Snapshot() =>
        _registrations.Values
            .OrderBy(registration => registration.Rule.Identity.RuleId, StringComparer.Ordinal)
            .ThenBy(registration => registration.Scope)
            .ThenBy(
                registration => registration.ScopeKey ?? string.Empty,
                StringComparer.Ordinal)
            .ThenBy(
                registration => registration.Rule.Identity.ImplementationVersion,
                StringComparer.Ordinal)
            .ToArray();

    private static bool IsApplicable(
        EngineeringRuleRegistration registration,
        EngineeringRuleResolutionContext context) =>
        registration.Scope switch
        {
            EngineeringRuleScope.Default => true,
            EngineeringRuleScope.Company =>
                string.Equals(
                    registration.ScopeKey,
                    context.CompanyId,
                    StringComparison.Ordinal),
            EngineeringRuleScope.Project =>
                string.Equals(
                    registration.ScopeKey,
                    context.ProjectId,
                    StringComparison.Ordinal),
            EngineeringRuleScope.Part =>
                string.Equals(
                    registration.ScopeKey,
                    context.PartId,
                    StringComparison.Ordinal),
            EngineeringRuleScope.Process =>
                string.Equals(
                    registration.ScopeKey,
                    context.ProcessId,
                    StringComparison.Ordinal),
            _ => false
        };

    private readonly record struct RuleRegistrationKey(
        string RuleId,
        EngineeringRuleScope Scope,
        string? ScopeKey);
}
