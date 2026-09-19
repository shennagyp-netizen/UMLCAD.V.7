using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Science;
namespace UMLCAD.Engineering.Runtime;
public readonly record struct EngineeringRuleId(string Value);
public enum RuleOutcome{Pass,Warn,Reject,Indeterminate,Failed}
public sealed record EngineeringContext(CadEvaluationSnapshot Cad,IPhenomenaSimulationService Simulation);
public sealed record EngineeringRuleResult(EngineeringRuleId RuleId,RuleOutcome Outcome,IReadOnlyList<CadDiagnostic> Diagnostics);
public interface IEngineeringRule{EngineeringRuleId Id{get;}ValueTask<EngineeringRuleResult> EvaluateAsync(EngineeringContext context,CancellationToken cancellationToken=default);}
public sealed class EngineeringRuleRuntime
{
    public async ValueTask<IReadOnlyList<EngineeringRuleResult>> EvaluateAsync(EngineeringContext context,IEnumerable<IEngineeringRule> rules,CancellationToken cancellationToken=default)
    {
        var output=new List<EngineeringRuleResult>();
        foreach(var rule in rules.OrderBy(x=>x.Id.Value,StringComparer.Ordinal))
        {cancellationToken.ThrowIfCancellationRequested();var r=await rule.EvaluateAsync(context,cancellationToken);output.Add(r);if(r.Outcome is RuleOutcome.Reject or RuleOutcome.Failed)break;}
        return output;
    }
}
