using System.Globalization;
using System.Security.Cryptography;
using System.Text;

namespace UMLCAD.Cad.Expressions;

public abstract record CadExpression
{
    public abstract double Evaluate(IReadOnlyDictionary<string, double> parameters);
    public abstract string CanonicalForm { get; }

    public string Identity =>
        Convert.ToHexString(
            SHA256.HashData(Encoding.UTF8.GetBytes(CanonicalForm)))
        .ToLowerInvariant();

    public static CadExpression Constant(double value) =>
        new ConstantExpression(value);

    public static CadExpression Parameter(string name) =>
        new ParameterExpression(name);

    public static CadExpression Add(params CadExpression[] terms) =>
        new SumExpression(terms);
}

public sealed record ConstantExpression : CadExpression
{
    public double Value { get; }

    public ConstantExpression(double value)
    {
        if (!double.IsFinite(value))
            throw new ArgumentOutOfRangeException(nameof(value));

        Value = value;
    }

    public override double Evaluate(
        IReadOnlyDictionary<string, double> parameters) => Value;

    public override string CanonicalForm =>
        "const:" + Value.ToString("R", CultureInfo.InvariantCulture);
}

public sealed record ParameterExpression : CadExpression
{
    public string Name { get; }

    public ParameterExpression(string name)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Parameter is required.", nameof(name));

        Name = name;
    }

    public override double Evaluate(
        IReadOnlyDictionary<string, double> parameters)
    {
        if (!parameters.TryGetValue(Name, out var value))
            throw new KeyNotFoundException(Name);

        return value;
    }

    public override string CanonicalForm => "param:" + Name;
}

public sealed record SumExpression : CadExpression
{
    public IReadOnlyList<CadExpression> Terms { get; }

    public SumExpression(IEnumerable<CadExpression> terms)
    {
        ArgumentNullException.ThrowIfNull(terms);

        Terms = terms.ToArray();

        if (Terms.Count == 0)
            throw new ArgumentException("Sum needs terms.", nameof(terms));

        if (Terms.Any(term => term is null))
            throw new ArgumentException(
                "Sum terms cannot contain null expressions.",
                nameof(terms));
    }

    public override double Evaluate(
        IReadOnlyDictionary<string, double> parameters) =>
        Terms.Sum(term => term.Evaluate(parameters));

    public override string CanonicalForm =>
        "sum(" + string.Join(",", Terms.Select(term => term.CanonicalForm)) + ")";
}

public sealed record CadParameterBinding(
    string Name,
    CadExpression Expression,
    string? Unit = null);
