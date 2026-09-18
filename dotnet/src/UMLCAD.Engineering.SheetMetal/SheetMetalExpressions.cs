using UMLCAD.Cad.Expressions;

namespace UMLCAD.Engineering.SheetMetal;

public static class SheetMetalExpressions
{
    public static ExpressionNode BendAllowance()
    {
        // Canonical target:
        // (((pi / 180) * (R + (K * T))) * A)
        return new BinaryExpression(
            BinaryOperator.Multiply,
            new BinaryExpression(
                BinaryOperator.Multiply,
                new BinaryExpression(
                    BinaryOperator.Divide,
                    new VariableExpression("pi"),
                    new ConstantExpression(180d)),
                new BinaryExpression(
                    BinaryOperator.Add,
                    new VariableExpression("R"),
                    new BinaryExpression(
                        BinaryOperator.Multiply,
                        new VariableExpression("K"),
                        new VariableExpression("T")))),
            new VariableExpression("A"));
    }
}
