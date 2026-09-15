namespace UMLCAD.Framework.Semantics;

public interface IPartSemanticService
{
    PartSemantic? Get(string id);
    IReadOnlyList<PartSemantic> GetAll();
}

public interface IDrawingSemanticService
{
    DrawingSemantic? Get(string id);
    IReadOnlyList<DrawingSemantic> GetAll();
}

public interface ISheetSemanticService
{
    SheetSemantic? Get(string drawingId, string sheetId);
}

public interface IAssemblySemanticService
{
    AssemblySemantic? Get(string id);
    IReadOnlyList<AssemblySemantic> GetAll();
}

public interface ISemanticApplication
{
    SemanticApplication Current { get; }
}

internal sealed class SemanticApplicationState : ISemanticApplication
{
    private SemanticApplication? _current;

    public SemanticApplication Current =>
        _current ?? throw new InvalidOperationException("The CAD application has not completed Build().");

    public void SetCurrent(SemanticApplication application)
    {
        ArgumentNullException.ThrowIfNull(application);
        _current = application;
    }
}

internal sealed class PartSemanticService : IPartSemanticService
{
    private readonly ISemanticApplication _application;

    public PartSemanticService(ISemanticApplication application) => _application = application;

    public PartSemantic? Get(string id) =>
        _application.Current.Parts.FirstOrDefault(x => string.Equals(x.Id, id, StringComparison.Ordinal));

    public IReadOnlyList<PartSemantic> GetAll() => _application.Current.Parts;
}

internal sealed class DrawingSemanticService : IDrawingSemanticService
{
    private readonly ISemanticApplication _application;

    public DrawingSemanticService(ISemanticApplication application) => _application = application;

    public DrawingSemantic? Get(string id) =>
        _application.Current.Drawings.FirstOrDefault(x => string.Equals(x.Id, id, StringComparison.Ordinal));

    public IReadOnlyList<DrawingSemantic> GetAll() => _application.Current.Drawings;
}

internal sealed class SheetSemanticService : ISheetSemanticService
{
    private readonly ISemanticApplication _application;

    public SheetSemanticService(ISemanticApplication application) => _application = application;

    public SheetSemantic? Get(string drawingId, string sheetId) =>
        _application.Current.Drawings
            .FirstOrDefault(x => string.Equals(x.Id, drawingId, StringComparison.Ordinal))?
            .Sheets.FirstOrDefault(x => string.Equals(x.Id, sheetId, StringComparison.Ordinal));
}

internal sealed class AssemblySemanticService : IAssemblySemanticService
{
    private readonly ISemanticApplication _application;

    public AssemblySemanticService(ISemanticApplication application) => _application = application;

    public AssemblySemantic? Get(string id) =>
        _application.Current.Assemblies.FirstOrDefault(x => string.Equals(x.Id, id, StringComparison.Ordinal));

    public IReadOnlyList<AssemblySemantic> GetAll() => _application.Current.Assemblies;
}
