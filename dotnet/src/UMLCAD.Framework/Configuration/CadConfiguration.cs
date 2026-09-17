using Microsoft.Extensions.Configuration;

namespace UMLCAD.Framework.Configuration;

public interface ICadConfiguration
{
    IConfiguration Current { get; }
}

internal sealed class CadConfiguration(IConfiguration configuration) : ICadConfiguration
{
    public IConfiguration Current { get; } = configuration;
}
