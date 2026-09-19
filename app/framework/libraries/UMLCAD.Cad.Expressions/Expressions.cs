using System.Globalization;
using System.Security.Cryptography;
using System.Text;
namespace UMLCAD.Cad.Expressions;
public abstract record CadExpression
{
    public abstract double Evaluate(IReadOnlyDictionary<string,double> parameters);
    public abstract string CanonicalForm{get;}
    public string Identity=>Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(CanonicalForm))).ToLowerInvariant();
    public static CadExpression Constant(double value)=>new ConstantExpression(value);
    public static CadExpression Parameter(string name)=>new ParameterExpression(name);
    public static CadExpression Add(params CadExpression[] terms)=>new SumExpression(terms);
}
public sealed record ConstantExpression(double Value):CadExpression
{
    public ConstantExpression{if(!double.IsFinite(Value))throw new ArgumentOutOfRangeException(nameof(Value));}
    public override double Evaluate(IReadOnlyDictionary<string,double> parameters)=>Value;
    public override string CanonicalForm=>"const:"+Value.ToString("R",CultureInfo.InvariantCulture);
}
public sealed record ParameterExpression(string Name):CadExpression
{
    public ParameterExpression{if(string.IsNullOrWhiteSpace(Name))throw new ArgumentException("Parameter is required.");}
    public override double Evaluate(IReadOnlyDictionary<string,double> parameters){if(!parameters.TryGetValue(Name,out var value))throw new KeyNotFoundException(Name);return value;}
    public override string CanonicalForm=>"param:"+Name;
}
public sealed record SumExpression(IReadOnlyList<CadExpression> Terms):CadExpression
{
    public SumExpression(IEnumerable<CadExpression> terms):this(terms.ToArray()){}
    public SumExpression{if(Terms.Count==0)throw new ArgumentException("Sum needs terms.");}
    public override double Evaluate(IReadOnlyDictionary<string,double> parameters)=>Terms.Sum(x=>x.Evaluate(parameters));
    public override string CanonicalForm=>"sum("+string.Join(",",Terms.Select(x=>x.CanonicalForm))+")";
}
public sealed record CadParameterBinding(string Name,CadExpression Expression,string? Unit=null);