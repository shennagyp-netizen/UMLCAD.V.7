using System.Security.Cryptography;
using System.Text.Json;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework.Configuration;
using UMLCAD.Framework.Semantics;

namespace UMLCAD.Framework;

public sealed partial class CadApplication
{
    public static CadApplicationBuilder CreateBuilder() => new();
}

public sealed class CadApplicationBuilder
{
    private readonly IServiceCollection _userServices = new ServiceCollection();
    private readonly ConfigurationManager _configuration = new();
    private readonly BuildHistory _buildHistory = new();
    private string _applicationId = "uml-cad-application";
    private string _version = "1.0.0";
    private readonly List<PartDefinition> _parts = [];
    private readonly List<AssemblyDefinition> _assemblies = [];
    private readonly List<DrawingDefinition> _drawings = [];

    public IServiceCollection Services => _userServices;
    public IConfigurationManager Configuration => _configuration;

    public string ApplicationId
    {
        get => _applicationId;
        set => _applicationId = RequireId(value, nameof(value));
    }

    public string Version
    {
        get => _version;
        set => _version = RequireId(value, nameof(value));
    }

    public CadApplicationBuilder AddPart(string id, string partType, Action<PartBuilder>? configure = null)
    {
        var builder = new PartBuilder(RequireText(id, nameof(id)), RequireText(partType, nameof(partType)));
        configure?.Invoke(builder);
        _parts.Add(builder.Build());
        return this;
    }

    public CadApplicationBuilder AddAssembly(string id, string name, Action<AssemblyBuilder>? configure = null)
    {
        var builder = new AssemblyBuilder(RequireText(id, nameof(id)), RequireText(name, nameof(name)));
        configure?.Invoke(builder);
        _assemblies.Add(builder.Build());
        return this;
    }

    public CadApplicationBuilder AddDrawing(string id, string name, Action<DrawingBuilder>? configure = null)
    {
        var builder = new DrawingBuilder(RequireText(id, nameof(id)), RequireText(name, nameof(name)));
        configure?.Invoke(builder);
        _drawings.Add(builder.Build());
        return this;
    }

    public CadApplication Build()
    {
        ValidateDefinitions();

        var services = new ServiceCollection();
        foreach (var descriptor in _userServices)
            ((ICollection<ServiceDescriptor>)services).Add(descriptor);

        var semanticState = new SemanticApplicationState();
        services.AddSingleton<IConfiguration>(_configuration);
        services.AddSingleton<ICadConfiguration>(_ => new CadConfiguration(_configuration));
        services.AddSingleton<IBuildHistory>(_buildHistory);
        services.AddSingleton<ISemanticApplication>(semanticState);
        services.AddSingleton<ISemanticReferenceService, SemanticReferenceService>();
        services.AddSingleton<IPartSemanticService, PartSemanticService>();
        services.AddSingleton<IDrawingSemanticService, DrawingSemanticService>();
        services.AddSingleton<ISheetSemanticService, SheetSemanticService>();
        services.AddSingleton<IAssemblySemanticService, AssemblySemanticService>();
        services.AddSingleton<IBuildPackageService, BuildPackageService>();
        services.AddSingleton<ISemanticRegistry, SemanticRegistry>();
        services.AddSingleton<ICompiledModelService, CompiledModelService>();

        var provider = services.BuildServiceProvider(new ServiceProviderOptions
        {
            ValidateScopes = true,
            ValidateOnBuild = true
        });

        try
        {
            var registry = provider.GetRequiredService<ISemanticRegistry>();
            foreach (var part in _parts)
                registry.RegisterPart(part.ToSemantic());
            foreach (var assembly in _assemblies)
                registry.RegisterAssembly(assembly.ToSemantic());
            foreach (var drawing in _drawings)
                registry.RegisterDrawing(drawing.ToSemantic());

            var configuration = ResolveConfiguration();
            var canonicalWithoutIdentity = new
            {
                Schema = "uml-cad-semantic/1.2.0",
                ApplicationId,
                Version,
                Configuration = configuration,
                Parts = _parts.OrderBy(x => x.Id, StringComparer.Ordinal),
                Assemblies = _assemblies.OrderBy(x => x.Id, StringComparer.Ordinal),
                Drawings = _drawings.OrderBy(x => x.Id, StringComparer.Ordinal)
            };

            var bytes = JsonSerializer.SerializeToUtf8Bytes(canonicalWithoutIdentity, JsonDefaults.Options);
            var identity = Convert.ToHexString(SHA256.HashData(bytes)).ToLowerInvariant();
            var semantic = registry.Snapshot(ApplicationId, Version, configuration, identity);
            semanticState.SetCurrent(semantic);
            _buildHistory.Record(semantic, identity);

            return new CadApplication(semantic, provider, _configuration);
        }
        catch
        {
            provider.Dispose();
            throw;
        }
    }

    private void ValidateDefinitions()
    {
        var definitionIds = new HashSet<string>(StringComparer.Ordinal);
        foreach (var part in _parts)
            if (!definitionIds.Add(part.Id))
                throw new InvalidOperationException($"A semantic definition with ID '{part.Id}' is already defined.");
        foreach (var assembly in _assemblies)
            if (!definitionIds.Add(assembly.Id))
                throw new InvalidOperationException($"A semantic definition with ID '{assembly.Id}' is already defined.");
        foreach (var drawing in _drawings)
            if (!definitionIds.Add(drawing.Id))
                throw new InvalidOperationException($"A semantic definition with ID '{drawing.Id}' is already defined.");

        if (definitionIds.Contains(ApplicationId))
            throw new InvalidOperationException($"Application ID '{ApplicationId}' collides with a semantic definition ID.");

        var assemblyLookup = _assemblies.ToDictionary(x => x.Id, StringComparer.Ordinal);
        var partLookup = _parts.ToDictionary(x => x.Id, StringComparer.Ordinal);

        foreach (var drawing in _drawings)
        {
            foreach (var partReference in drawing.PartReferences)
                if (!partLookup.ContainsKey(partReference))
                    throw new InvalidOperationException($"Drawing '{drawing.Id}' references unknown part definition '{partReference}'.");
        }

        foreach (var assembly in _assemblies)
        {
            var localOccurrenceIds = new HashSet<string>(StringComparer.Ordinal);
            foreach (var occurrence in assembly.Occurrences)
            {
                if (!localOccurrenceIds.Add(occurrence.Id))
                    throw new InvalidOperationException($"Assembly '{assembly.Id}' contains duplicate occurrence ID '{occurrence.Id}'.");

                var exists = occurrence.DefinitionKind switch
                {
                    "part" => partLookup.ContainsKey(occurrence.DefinitionId),
                    "assembly" => assemblyLookup.ContainsKey(occurrence.DefinitionId),
                    _ => false
                };

                if (!exists)
                    throw new InvalidOperationException($"Assembly occurrence '{assembly.Id}/{occurrence.Id}' references unknown {occurrence.DefinitionKind} definition '{occurrence.DefinitionId}'.");
            }
        }

        var visiting = new HashSet<string>(StringComparer.Ordinal);
        var visited = new HashSet<string>(StringComparer.Ordinal);
        foreach (var assembly in _assemblies)
            VisitAssembly(assembly.Id, assemblyLookup, visiting, visited);
    }

    private static void VisitAssembly(
        string assemblyId,
        IReadOnlyDictionary<string, AssemblyDefinition> assemblies,
        HashSet<string> visiting,
        HashSet<string> visited)
    {
        if (visited.Contains(assemblyId))
            return;
        if (!visiting.Add(assemblyId))
            throw new InvalidOperationException($"Circular assembly reference detected at '{assemblyId}'.");

        var assembly = assemblies[assemblyId];
        foreach (var occurrence in assembly.Occurrences.Where(x => x.DefinitionKind == "assembly"))
            VisitAssembly(occurrence.DefinitionId, assemblies, visiting, visited);

        visiting.Remove(assemblyId);
        visited.Add(assemblyId);
    }

    private IReadOnlyDictionary<string, string> ResolveConfiguration() =>
        new SortedDictionary<string, string>(
            _configuration.AsEnumerable()
                .Where(x => x.Value is not null)
                .ToDictionary(x => x.Key, x => x.Value!, StringComparer.Ordinal),
            StringComparer.Ordinal);

    private static string RequireId(string value, string parameterName) => RequireText(value, parameterName);

    private static string RequireText(string value, string parameterName)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Value cannot be empty.", parameterName);
        return value;
    }

    private static class JsonDefaults
    {
        public static readonly JsonSerializerOptions Options = new(JsonSerializerDefaults.General)
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false
        };
    }
}

public sealed partial class CadApplication : IDisposable
{
    private readonly ServiceProvider _services;

    internal CadApplication(
        SemanticApplication semantic,
        ServiceProvider services,
        IConfiguration configuration)
    {
        Semantic = semantic;
        _services = services;
        Configuration = configuration;
    }

    public SemanticApplication Semantic { get; }
    public IConfiguration Configuration { get; }
    public IServiceProvider Services => _services;

    public T GetRequiredService<T>() where T : notnull => _services.GetRequiredService<T>();

    public BuildPackage CreateBuildPackage() =>
        GetRequiredService<IBuildPackageService>().CreatePackage(Semantic);

    public CompiledModelManifest CreateCompiledModelManifest() =>
        GetRequiredService<ICompiledModelService>().Create(Semantic);

    public void Dispose() => _services.Dispose();
}

public sealed class PartBuilder
{
    private readonly string _id;
    private readonly string _partType;
    private readonly List<ParameterSemantic> _parameters = [];
    private readonly List<GeometrySemantic> _geometry = [];
    private readonly List<ConstraintSemantic> _constraints = [];
    private readonly List<string> _references = [];
    private readonly List<string> _components = [];
    private readonly List<ShapePublicationSemantic> _publications = [];
    private readonly List<TopologyBindingSemantic> _topologyBindings = [];
    private readonly Dictionary<string, string> _customMetadata = new(StringComparer.Ordinal);
    private string _name;
    private string? _partNumber;
    private string? _description;
    private string? _material;
    private string? _manufacturer;
    private string? _vendor;
    private string? _revision;
    private string? _lifecycleState;
    private string? _author;
    private string? _documentCode;

    internal PartBuilder(string id, string partType)
    {
        _id = id;
        _partType = partType;
        _name = id;
    }

    public PartBuilder Name(string value) { _name = RequireText(value, nameof(value)); return this; }
    public PartBuilder PartNumber(string value) { _partNumber = RequireText(value, nameof(value)); return this; }
    public PartBuilder Description(string value) { _description = RequireText(value, nameof(value)); return this; }
    public PartBuilder Material(string value) { _material = RequireText(value, nameof(value)); return this; }
    public PartBuilder Manufacturer(string value) { _manufacturer = RequireText(value, nameof(value)); return this; }
    public PartBuilder Vendor(string value) { _vendor = RequireText(value, nameof(value)); return this; }
    public PartBuilder Revision(string value) { _revision = RequireText(value, nameof(value)); return this; }
    public PartBuilder LifecycleState(string value) { _lifecycleState = RequireText(value, nameof(value)); return this; }
    public PartBuilder Author(string value) { _author = RequireText(value, nameof(value)); return this; }
    public PartBuilder DocumentCode(string value) { _documentCode = RequireText(value, nameof(value)); return this; }

    public PartBuilder Property(string name, string value)
    {
        _customMetadata[RequireText(name, nameof(name))] = value;
        return this;
    }

    public PartBuilder Parameter(string name, string value, string? unit = null)
    {
        _parameters.Add(new ParameterSemantic(RequireText(name, nameof(name)), RequireText(value, nameof(value)), unit));
        return this;
    }

    public PartBuilder Geometry(string id, string kind, IReadOnlyDictionary<string, string> properties)
    {
        _geometry.Add(new GeometrySemantic(RequireText(id, nameof(id)), RequireText(kind, nameof(kind)), ToSortedDictionary(properties)));
        return this;
    }

    public PartBuilder Constraint(string id, string kind, IReadOnlyList<string> references, IReadOnlyDictionary<string, string> properties)
    {
        _constraints.Add(new ConstraintSemantic(RequireText(id, nameof(id)), RequireText(kind, nameof(kind)), references.ToArray(), ToSortedDictionary(properties)));
        return this;
    }

    public PartBuilder Reference(string id)
    {
        _references.Add(RequireText(id, nameof(id)));
        return this;
    }

    public PartBuilder Component(string id)
    {
        _components.Add(RequireText(id, nameof(id)));
        return this;
    }

    public PartBuilder Publication(
        string id,
        string targetKind,
        string targetId,
        string resultIdentity,
        string topologyBindingId)
    {
        _publications.Add(new ShapePublicationSemantic(
            RequireText(id, nameof(id)),
            RequireText(targetKind, nameof(targetKind)),
            RequireText(targetId, nameof(targetId)),
            RequireText(resultIdentity, nameof(resultIdentity)),
            RequireText(topologyBindingId, nameof(topologyBindingId))));
        return this;
    }

    public PartBuilder TopologyBinding(
        string id,
        string topologyKind,
        string semanticTargetId,
        string resultIdentity,
        string authoritativeTopologyId)
    {
        _topologyBindings.Add(new TopologyBindingSemantic(
            RequireText(id, nameof(id)),
            RequireText(topologyKind, nameof(topologyKind)),
            RequireText(semanticTargetId, nameof(semanticTargetId)),
            RequireText(resultIdentity, nameof(resultIdentity)),
            RequireText(authoritativeTopologyId, nameof(authoritativeTopologyId))));
        return this;
    }

    internal PartDefinition Build() => new(
        _id,
        _name,
        _partType,
        _parameters.OrderBy(x => x.Name, StringComparer.Ordinal).ToArray(),
        _geometry.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
        _constraints.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
        _references.OrderBy(x => x, StringComparer.Ordinal).ToArray(),
        _components.OrderBy(x => x, StringComparer.Ordinal).ToArray(),
        _publications.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
        _topologyBindings.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
        new CadMetadata(_partNumber, _description, _material, _manufacturer, _vendor, _revision, _lifecycleState, _author, _documentCode,
            new SortedDictionary<string, string>(_customMetadata, StringComparer.Ordinal)));

    private static IReadOnlyDictionary<string, string> ToSortedDictionary(IReadOnlyDictionary<string, string> source) =>
        new SortedDictionary<string, string>(source.ToDictionary(x => x.Key, x => x.Value, StringComparer.Ordinal), StringComparer.Ordinal);

    private static string RequireText(string value, string parameterName)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Value cannot be empty.", parameterName);
        return value;
    }
}

public sealed class DrawingBuilder(string id, string name)
{
    private readonly List<SheetSemantic> _sheets = [];
    private readonly List<string> _parts = [];
    private readonly Dictionary<string, string> _settings = new(StringComparer.Ordinal);

    public DrawingBuilder Setting(string name, string value)
    {
        _settings[RequireText(name, nameof(name))] = value;
        return this;
    }

    public DrawingBuilder Sheet(string id, string name, IReadOnlyList<string>? drawingReferences = null)
    {
        _sheets.Add(new SheetSemantic(RequireText(id, nameof(id)), RequireText(name, nameof(name)),
            (drawingReferences ?? Array.Empty<string>()).ToArray(), new SortedDictionary<string, string>(StringComparer.Ordinal)));
        return this;
    }

    public DrawingBuilder PartReference(string partId)
    {
        _parts.Add(RequireText(partId, nameof(partId)));
        return this;
    }

    internal DrawingDefinition Build() => new(
        id,
        name,
        _sheets.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
        _parts.OrderBy(x => x, StringComparer.Ordinal).ToArray(),
        new SortedDictionary<string, string>(_settings, StringComparer.Ordinal));

    private static string RequireText(string value, string parameterName)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Value cannot be empty.", parameterName);
        return value;
    }
}

public sealed class AssemblyBuilder(string id, string name)
{
    private readonly List<AssemblyOccurrenceSemantic> _occurrences = [];
    private readonly Dictionary<string, string> _settings = new(StringComparer.Ordinal);
    private readonly Dictionary<string, string> _metadata = new(StringComparer.Ordinal);
    private string? _partNumber;
    private string? _description;
    private string? _manufacturer;
    private string? _vendor;
    private string? _revision;
    private string? _lifecycleState;
    private string? _author;
    private string? _documentCode;

    public AssemblyBuilder Part(string occurrenceId, string partId, string? name = null, Action<AssemblyOccurrenceBuilder>? configure = null) =>
        Occurrence(occurrenceId, name ?? partId, partId, "part", configure);

    public AssemblyBuilder Assembly(string occurrenceId, string assemblyId, string? name = null, Action<AssemblyOccurrenceBuilder>? configure = null) =>
        Occurrence(occurrenceId, name ?? assemblyId, assemblyId, "assembly", configure);

    public AssemblyBuilder Occurrence(string occurrenceId, string name, string definitionId, string definitionKind, Action<AssemblyOccurrenceBuilder>? configure = null)
    {
        var builder = new AssemblyOccurrenceBuilder(RequireText(occurrenceId, nameof(occurrenceId)), RequireText(name, nameof(name)), RequireText(definitionId, nameof(definitionId)), RequireText(definitionKind, nameof(definitionKind)));
        configure?.Invoke(builder);
        _occurrences.Add(builder.Build());
        return this;
    }

    public AssemblyBuilder PartNumber(string value) { _partNumber = RequireText(value, nameof(value)); return this; }
    public AssemblyBuilder Description(string value) { _description = RequireText(value, nameof(value)); return this; }
    public AssemblyBuilder Manufacturer(string value) { _manufacturer = RequireText(value, nameof(value)); return this; }
    public AssemblyBuilder Vendor(string value) { _vendor = RequireText(value, nameof(value)); return this; }
    public AssemblyBuilder Revision(string value) { _revision = RequireText(value, nameof(value)); return this; }
    public AssemblyBuilder LifecycleState(string value) { _lifecycleState = RequireText(value, nameof(value)); return this; }
    public AssemblyBuilder Author(string value) { _author = RequireText(value, nameof(value)); return this; }
    public AssemblyBuilder DocumentCode(string value) { _documentCode = RequireText(value, nameof(value)); return this; }

    public AssemblyBuilder Property(string name, string value)
    {
        _metadata[RequireText(name, nameof(name))] = value;
        return this;
    }

    public AssemblyBuilder Setting(string name, string value)
    {
        _settings[RequireText(name, nameof(name))] = value;
        return this;
    }

    internal AssemblyDefinition Build() => new(
        id,
        name,
        _occurrences.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
        new SortedDictionary<string, string>(_settings, StringComparer.Ordinal),
        new CadMetadata(_partNumber, _description, null, _manufacturer, _vendor, _revision, _lifecycleState, _author, _documentCode,
            new SortedDictionary<string, string>(_metadata, StringComparer.Ordinal)));

    private static string RequireText(string value, string parameterName)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Value cannot be empty.", parameterName);
        return value;
    }
}

public sealed class AssemblyOccurrenceBuilder(string id, string name, string definitionId, string definitionKind)
{
    private readonly Dictionary<string, string> _metadata = new(StringComparer.Ordinal);
    private TransformSemantic _transform = TransformSemantic.Identity;
    private string? _configurationName;
    private double _quantity = 1;
    private string? _bomStructure;
    private bool _visible = true;
    private bool _suppressed;
    private bool _grounded;
    private bool _flexible;

    public AssemblyOccurrenceBuilder Transform(IReadOnlyList<double> matrix)
    {
        _transform = TransformSemantic.FromArray(matrix);
        return this;
    }

    public AssemblyOccurrenceBuilder Configuration(string value) { _configurationName = RequireText(value, nameof(value)); return this; }
    public AssemblyOccurrenceBuilder Quantity(double value)
    {
        if (value <= 0 || double.IsNaN(value) || double.IsInfinity(value))
            throw new ArgumentOutOfRangeException(nameof(value));
        _quantity = value;
        return this;
    }
    public AssemblyOccurrenceBuilder BomStructure(string value) { _bomStructure = RequireText(value, nameof(value)); return this; }
    public AssemblyOccurrenceBuilder Visible(bool value = true) { _visible = value; return this; }
    public AssemblyOccurrenceBuilder Suppressed(bool value = true) { _suppressed = value; return this; }
    public AssemblyOccurrenceBuilder Grounded(bool value = true) { _grounded = value; return this; }
    public AssemblyOccurrenceBuilder Flexible(bool value = true) { _flexible = value; return this; }

    public AssemblyOccurrenceBuilder Property(string name, string value)
    {
        _metadata[RequireText(name, nameof(name))] = value;
        return this;
    }

    internal AssemblyOccurrenceSemantic Build() => new(
        id, name, definitionId, definitionKind, _transform,
        new SortedDictionary<string, string>(_metadata, StringComparer.Ordinal),
        _configurationName, _quantity, _bomStructure, _visible, _suppressed, _grounded, _flexible);

    private static string RequireText(string value, string parameterName)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Value cannot be empty.", parameterName);
        return value;
    }
}

internal sealed record PartDefinition(
    string Id,
    string Name,
    string PartType,
    IReadOnlyList<ParameterSemantic> Parameters,
    IReadOnlyList<GeometrySemantic> Geometry,
    IReadOnlyList<ConstraintSemantic> Constraints,
    IReadOnlyList<string> References,
    IReadOnlyList<string> Components,
    IReadOnlyList<ShapePublicationSemantic> Publications,
    IReadOnlyList<TopologyBindingSemantic> TopologyBindings,
    CadMetadata Metadata)
{
    public PartSemantic ToSemantic() => new(Id, PartType, Parameters, Geometry, Constraints, References, Components)
    {
        Name = Name,
        StructuredMetadata = Metadata,
        Metadata = Metadata,
        Publications = Publications,
        TopologyBindings = TopologyBindings
    };
}

internal sealed record DrawingDefinition(
    string Id,
    string Name,
    IReadOnlyList<SheetSemantic> Sheets,
    IReadOnlyList<string> PartReferences,
    IReadOnlyDictionary<string, string> Settings)
{
    public DrawingSemantic ToSemantic() => new(Id, Name, Sheets, PartReferences, Settings);
}

internal sealed record AssemblyDefinition(
    string Id,
    string Name,
    IReadOnlyList<AssemblyOccurrenceSemantic> Occurrences,
    IReadOnlyDictionary<string, string> Settings,
    CadMetadata Metadata)
{
    public AssemblySemantic ToSemantic() => new(Id, Name, Array.Empty<string>(), Settings)
    {
        Occurrences = Occurrences,
        StructuredMetadata = Metadata,
        Metadata = Metadata
    };
}
