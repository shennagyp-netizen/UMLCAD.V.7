namespace UMLCAD.Cad.Expressions;

public enum UnaryOperator
{
    Negate,
}

public enum BinaryOperator
{
    Add,
    Subtract,
    Multiply,
    Divide,
}

public abstract record ExpressionNode
{
    public abstract double Evaluate(IReadOnlyDictionary<string, double> variables);

    public abstract string ToCanonicalString();
}

public sealed record ConstantExpression(double Value) : ExpressionNode
{
    public override double Evaluate(IReadOnlyDictionary<string, double> variables)
    {
        if (!double.IsFinite(Value))
            throw new ArgumentOutOfRangeException(
                nameof(Value),
                "Expression constants must be finite.");

        return Value;
    }

    public override string ToCanonicalString() =>
        Value.ToString(
            "R",
            System.Globalization.CultureInfo.InvariantCulture);
}

public sealed record VariableExpression : ExpressionNode
{
    public string Name { get; }

    public VariableExpression(string name)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException(
                "Expression variable names cannot be empty.",
                nameof(name));

        Name = name;
    }

    public override double Evaluate(
        IReadOnlyDictionary<string, double> variables)
    {
        if (!variables.TryGetValue(Name, out var value))
            throw new KeyNotFoundException(
                $"No value was supplied for expression variable '{Name}'.");

        if (!double.IsFinite(value))
            throw new ArgumentOutOfRangeException(
                nameof(variables),
                $"Expression variable '{Name}' is non-finite.");

        return value;
    }

    public override string ToCanonicalString() => Name;
}

public sealed record UnaryExpression(
    UnaryOperator Operator,
    ExpressionNode Operand) : ExpressionNode
{
    public override double Evaluate(
        IReadOnlyDictionary<string, double> variables)
    {
        var value = Operand.Evaluate(variables);

        return Operator switch
        {
            UnaryOperator.Negate => -value,
            _ => throw new InvalidOperationException(
                $"Unsupported unary operator: {Operator}.")
        };
    }

    public override string ToCanonicalString() =>
        Operator switch
        {
            UnaryOperator.Negate =>
                $"(-{Operand.ToCanonicalString()})",
            _ => throw new InvalidOperationException(
                $"Unsupported unary operator: {Operator}.")
        };
}

public sealed record BinaryExpression(
    BinaryOperator Operator,
    ExpressionNode Left,
    ExpressionNode Right) : ExpressionNode
{
    public override double Evaluate(
        IReadOnlyDictionary<string, double> variables)
    {
        var left = Left.Evaluate(variables);
        var right = Right.Evaluate(variables);

        var result = Operator switch
        {
            BinaryOperator.Add => left + right,
            BinaryOperator.Subtract => left - right,
            BinaryOperator.Multiply => left * right,
            BinaryOperator.Divide when right != 0d => left / right,
            BinaryOperator.Divide =>
                throw new DivideByZeroException(
                    "Expression division by zero."),
            _ => throw new InvalidOperationException(
                $"Unsupported binary operator: {Operator}.")
        };

        if (!double.IsFinite(result))
            throw new ArithmeticException(
                "Expression evaluation produced a non-finite result.");

        return result;
    }

    public override string ToCanonicalString()
    {
        var symbol = Operator switch
        {
            BinaryOperator.Add => "+",
            BinaryOperator.Subtract => "-",
            BinaryOperator.Multiply => "*",
            BinaryOperator.Divide => "/",
            _ => throw new InvalidOperationException(
                $"Unsupported binary operator: {Operator}.")
        };

        return $"({Left.ToCanonicalString()} {symbol} {Right.ToCanonicalString()})";
    }
}
