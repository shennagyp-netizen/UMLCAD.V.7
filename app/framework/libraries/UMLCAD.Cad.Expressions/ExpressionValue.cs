namespace UMLCAD.Cad.Expressions;

public readonly record struct ExpressionValue(double Value, string Unit)
{
    public ExpressionValue
    {
        if (!double.IsFinite(Value))
            throw new ArgumentOutOfRangeException(nameof(Value), "Expression value must be finite.");

        if (string.IsNullOrWhiteSpace(Unit))
            throw new ArgumentException("Expression unit cannot be empty.", nameof(Unit));
    }
}
