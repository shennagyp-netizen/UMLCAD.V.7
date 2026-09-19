using System.Globalization;
using System.Security.Cryptography;
using System.Text;

namespace UMLCAD.Cad.Expressions;

public abstract record CadExpression
{
    public abstract string CanonicalForm { get; }
    public abstract double Evaluate(IReadOnlyDictionary<string, double> values);
    public abstract IReadOnlySet<string> ParameterNames { get; }

    public string Identity =>
        "expr:" + Convert.ToHexString(
            SHA256.HashData(Encoding.UTF8.GetBytes(CanonicalForm)))
            .ToLowerInvariant();

    public static CadExpression Constant(double value) => new ConstantExpression(value);
    public static CadExpression Parameter(string name) => new ParameterExpression(name);
    public static CadExpression Add(params CadExpression[] values) => new SumExpression(values);
    public static CadExpression Multiply(params CadExpression[] values) => new ProductExpression(values);
}

public sealed record ConstantExpression(double Value) : CadExpression
{
    public ConstantExpression
    {
        if (!double.IsFinite(Value))
            throw new ArgumentOutOfRangeException(nameof(Value));
    }

    public override string CanonicalForm =>
        "const:" + Value.ToString("R", CultureInfo.InvariantCulture);

    public override double Evaluate(IReadOnlyDictionary<string, double> values) => Value;

    public override IReadOnlySet<string> ParameterNames { get; } =
        new HashSet<string>(StringComparer.Ordinal);
}

public sealed record ParameterExpression(string Name) : CadExpression
{
    public ParameterExpression
    {
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Parameter name is required.", nameof(Name));
    }

    public override string CanonicalForm => "param:" + Name;

    public override double Evaluate(IReadOnlyDictionary<string, double> values) =>
        values.TryGetValue(Name, out var value)
            ? value
            : throw new KeyNotFoundException($"Parameter '{Name}' is not resolved.");

    public override IReadOnlySet<string> ParameterNames { get; } =
        new HashSet<string>(new[] { Name }, StringComparer.Ordinal);
}

public sealed record SumExpression : CadExpression
{
    public IReadOnlyList<CadExpression> Terms { get; }

    public SumExpression(IEnumerable<CadExpression> terms)
    {
        ArgumentNullException.ThrowIfNull(terms);
        Terms = terms.ToArray();
        if (Terms.Count == 0)
            throw new ArgumentException("A sum requires terms.");
    }

    public override string CanonicalForm =>
        "sum(" + string.Join(",", Terms.Select(x => x.CanonicalForm)) + ")";

    public override double Evaluate(IReadOnlyDictionary<string, double> values) =>
        Terms.Sum(x => x.Evaluate(values));

    public override IReadOnlySet<string> ParameterNames { get; } =
        new HashSet<string>(StringComparer.Ordinal);

    public IReadOnlySet<string> Names =>
        Terms.SelectMany(x => x.ParameterNames)
            .ToHashSet(StringComparer.Ordinal);
}

public sealed record ProductExpression : CadExpression
{
    public IReadOnlyList<CadExpression> Factors { get; }

    public ProductExpression(IEnumerable<CadExpression> factors)
    {
        ArgumentNullException.ThrowIfNull(factors);
        Factors = factors.ToArray();
        if (Factors.Count == 0)
            throw new ArgumentException("A product requires factors.");
    }

    public override string CanonicalForm =>
        "mul(" + string.Join(",", Factors.Select(x => x.CanonicalForm)) + ")";

    public override double Evaluate(IReadOnlyDictionary<string, double> values) =>
        Factors.Aggregate(1d, (current, factor) => current * factor.Evaluate(values));

    public override IReadOnlySet<string> ParameterNames =>
        Factors.SelectMany(x => x.ParameterNames)
            .ToHashSet(StringComparer.Ordinal);
}

public sealed record CadParameter(
    string Name,
    CadExpression Expression,
    string? Unit = null)
{
    public CadParameter
    {
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Parameter name is required.", nameof(Name));
        ArgumentNullException.ThrowIfNull(Expression);
    }

    public string Identity =>
        $"{Name}|{Expression.Identity}|{Unit ?? "-"}";
}
