namespace UMLCAD.Engineering.Resources;
public sealed record MachineResource(string Id,string Name,string ProcessFamily);
public sealed record ToolResource(string Id,string Name,string ToolType,double DiameterMm);
public sealed record FixtureResource(string Id,string Name);
public sealed record ProcessCapability(string Id,string ProcessFamily,IReadOnlySet<string> MachineIds,IReadOnlySet<string> ToolTypes);