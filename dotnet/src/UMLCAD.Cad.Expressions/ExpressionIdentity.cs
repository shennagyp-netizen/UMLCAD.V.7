using System.Security.Cryptography;
using System.Text;

namespace UMLCAD.Cad.Expressions;

public static class ExpressionIdentity
{
    public static string Canonicalize(ExpressionNode expression)
    {
        ArgumentNullException.ThrowIfNull(expression);
        return expression.ToCanonicalString();
    }

    public static string Sha256(ExpressionNode expression)
    {
        var canonical = Canonicalize(expression);
        var bytes = Encoding.UTF8.GetBytes(canonical);
        return Convert.ToHexString(SHA256.HashData(bytes)).ToLowerInvariant();
    }
}
