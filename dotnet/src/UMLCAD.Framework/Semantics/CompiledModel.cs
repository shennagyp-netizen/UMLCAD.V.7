using System.Text.Json;

namespace UMLCAD.Framework.Semantics;

public sealed record CompiledModelPackage(
    string Schema,
    string ApplicationId,
    string ApplicationVersion,
    string BuildIdentity,
    CompiledModelManifest Manifest,
    CompiledRenderArtifact? RenderArtifact,
    IReadOnlyList<CompiledDiagnostic> Diagnostics);

public sealed record CompiledModelManifest(
    string Schema,
    string ApplicationId,
    string ApplicationVersion,
    string BuildIdentity,
    IReadOnlyList<string> RootNodeIds,
    IReadOnlyList<CompiledNode> Nodes,
    IReadOnlyList<CompiledRelationship> Relationships,
    IReadOnlyList<CompiledRepresentation> Representations,
    IReadOnlyList<CompiledTopologyBinding> TopologyBindings,
    IReadOnlyList<CompiledSourceBinding> SourceBindings,
    IReadOnlyList<CompiledDiagnostic> Diagnostics);

public sealed record CompiledNode(
    string Id,
    string Name,
    string Kind,
    string? ParentId,
    IReadOnlyList<string> ChildIds,
    IReadOnlyDictionary<string, JsonElement> Metadata,
    IReadOnlyList<string> RelationshipIds,
    IReadOnlyList<string> RepresentationIds,
    CompiledCapabilities Capabilities,
    CompiledSourceBinding? Source,
    IReadOnlyDictionary<string, string> State);

public sealed record CompiledRelationship(
    string Id,
    string Kind,
    string SourceId,
    IReadOnlyList<string> TargetIds,
    IReadOnlyDictionary<string, JsonElement> Metadata);

public sealed record CompiledRepresentation(
    string Id,
    string Kind,
    string ArtifactId,
    string? RenderNodeId,
    IReadOnlyList<double>? Bounds,
    IReadOnlyDictionary<string, string> Capabilities,
    IReadOnlyList<string> SelectableSubTargets);

public sealed record CompiledTopologyBinding(
    string Id,
    string TopologyKind,
    string SemanticNodeId,
    string ArtifactId,
    string RenderPrimitiveId,
    string? Name,
    string? Nomenclature,
    int? Number,
    IReadOnlyList<double>? Bounds,
    IReadOnlyDictionary<string, JsonElement> Metadata);

public sealed record CompiledSourceBinding(string File, string? Symbol, string? Start, string? End, string? Revision);
public sealed record CompiledCapabilities(bool Visible, bool Hideable, bool Selectable, bool Focusable);

public sealed record CompiledRenderArtifact(
    string ArtifactId,
    string BuildIdentity,
    string Format,
    string MediaType,
    string AssetIdentity,
    long? SizeBytes,
    string? IntegritySha256,
    string? Uri,
    string? InlineBase64,
    IReadOnlyDictionary<string, string> Properties);

public sealed record CompiledDiagnostic(string Code, string Severity, string Message, string? TargetId = null);

public interface ICompiledModelService
{
    CompiledModelManifest Create(SemanticApplication application);
}

internal sealed class CompiledModelService : ICompiledModelService
{
    private const string Schema = "uml-cad-compiled-model/1.1.0";
    private readonly ISemanticReferenceService _referenceService;

    public CompiledModelService(ISemanticReferenceService referenceService) => _referenceService = referenceService;

    public CompiledModelManifest Create(SemanticApplication application)
    {
        ArgumentNullException.ThrowIfNull(application);

        var nodes = new Dictionary<string, CompiledNode>(StringComparer.Ordinal);
        var relationships = new Dictionary<string, CompiledRelationship>(StringComparer.Ordinal);

        foreach (var part in application.Parts.OrderBy(x => x.Id, StringComparer.Ordinal))
        {
            var definitionId = DefinitionId("Part", part.Id);
            var children = AddPartChildren(application, part, definitionId, nodes, relationships);
            AddNode(nodes, new CompiledNode(
                definitionId,
                part.Name,
                "Part",
                null,
                children,
                Metadata(part.StructuredMetadata),
                relationships.Values.Where(x => x.SourceId == definitionId).Select(x => x.Id).OrderBy(x => x, StringComparer.Ordinal).ToArray(),
                [],
                new CompiledCapabilities(true, true, true, true),
                null,
                new Dictionary<string, string>(StringComparer.Ordinal) { ["partType"] = part.PartType }));
        }

        foreach (var assembly in application.Assemblies.OrderBy(x => x.Id, StringComparer.Ordinal))
        {
            var definitionId = DefinitionId("Assembly", assembly.Id);
            var childIds = assembly.Occurrences
                .OrderBy(x => x.Id, StringComparer.Ordinal)
                .Select(x => AddOccurrence(assembly, x, definitionId, application, nodes, relationships))
                .ToArray();

            AddNode(nodes, new CompiledNode(
                definitionId,
                assembly.Name,
                "Assembly",
                null,
                childIds,
                Metadata(assembly.StructuredMetadata),
                relationships.Values.Where(x => x.SourceId == definitionId).Select(x => x.Id).OrderBy(x => x, StringComparer.Ordinal).ToArray(),
                [],
                new CompiledCapabilities(true, true, true, true),
                null,
                new Dictionary<string, string>(StringComparer.Ordinal)));
        }

        var referencedParts = new HashSet<string>(application.Assemblies.SelectMany(x => x.Occurrences.Where(o => o.DefinitionKind == "part").Select(o => o.DefinitionId)), StringComparer.Ordinal);
        var referencedAssemblies = new HashSet<string>(application.Assemblies.SelectMany(x => x.Occurrences.Where(o => o.DefinitionKind == "assembly").Select(o => o.DefinitionId)), StringComparer.Ordinal);
        var roots = new List<string>();
        roots.AddRange(application.Assemblies.Where(x => !referencedAssemblies.Contains(x.Id)).Select(x => DefinitionId("Assembly", x.Id)));
        roots.AddRange(application.Parts.Where(x => !referencedParts.Contains(x.Id)).Select(x => DefinitionId("Part", x.Id)));

        ValidateGraph(nodes, relationships, roots);

        return new CompiledModelManifest(
            Schema,
            application.Id,
            application.Version,
            application.BuildIdentity,
            roots.Distinct(StringComparer.Ordinal).OrderBy(x => x, StringComparer.Ordinal).ToArray(),
            nodes.Values.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
            relationships.Values.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
            [], [], [], []);
    }

    private IReadOnlyList<string> AddPartChildren(
        SemanticApplication application,
        PartSemantic part,
        string parentId,
        IDictionary<string, CompiledNode> nodes,
        IDictionary<string, CompiledRelationship> relationships)
    {
        var children = new List<string>();
        foreach (var geometry in part.Geometry.OrderBy(x => x.Id, StringComparer.Ordinal))
        {
            var nodeId = NodeChildId(parentId, "geometry", geometry.Id);
            AddNode(nodes, new CompiledNode(nodeId, geometry.Id, geometry.Kind, parentId, [], Metadata(geometry.Metadata), [], [],
                new CompiledCapabilities(true, true, true, true), null, new Dictionary<string, string>(StringComparer.Ordinal)));
            children.Add(nodeId);
        }

        var topologyBindings = part.TopologyBindings.ToDictionary(x => x.Id, StringComparer.Ordinal);
        foreach (var publication in part.Publications.OrderBy(x => x.Id, StringComparer.Ordinal))
        {
            var binding = topologyBindings[publication.TopologyBindingId];
            var nodeId = NodeChildId(parentId, "face", publication.Id);
            var metadata = new SortedDictionary<string, JsonElement>(StringComparer.Ordinal)
            {
                ["publicationId"] = JsonSerializer.SerializeToElement(publication.Id),
                ["targetId"] = JsonSerializer.SerializeToElement(publication.TargetId),
                ["targetKind"] = JsonSerializer.SerializeToElement(publication.TargetKind),
                ["resultIdentity"] = JsonSerializer.SerializeToElement(publication.ResultIdentity),
                ["topologyBindingId"] = JsonSerializer.SerializeToElement(publication.TopologyBindingId),
                ["authoritativeTopologyId"] = JsonSerializer.SerializeToElement(binding.AuthoritativeTopologyId)
            };

            AddNode(nodes, new CompiledNode(
                nodeId,
                publication.TargetId,
                "Face",
                parentId,
                [],
                metadata,
                [],
                [],
                new CompiledCapabilities(true, true, true, true),
                null,
                new Dictionary<string, string>(StringComparer.Ordinal)));

            children.Add(nodeId);
        }

        foreach (var constraint in part.Constraints.OrderBy(x => x.Id, StringComparer.Ordinal))
        {
            var nodeId = NodeChildId(parentId, "constraint", constraint.Id);
            AddNode(nodes, new CompiledNode(nodeId, constraint.Id, constraint.Kind, parentId, [], Metadata(constraint.Metadata), [], [],
                new CompiledCapabilities(false, false, true, false), null, new Dictionary<string, string>(StringComparer.Ordinal)));

            foreach (var reference in constraint.References.OrderBy(x => x, StringComparer.Ordinal))
            {
                var semanticReference = new SemanticReference(
                    part.Id,
                    reference,
                    "geometry");

                var resolution = _referenceService.Resolve(application, semanticReference);
                if (!resolution.IsResolved)
                {
                    var category = resolution.Status switch
                    {
                        SemanticReferenceStatus.Missing => "REFERENCE_MISSING",
                        SemanticReferenceStatus.Ambiguous => "REFERENCE_AMBIGUOUS",
                        SemanticReferenceStatus.Indeterminate => "REFERENCE_INDETERMINATE",
                        SemanticReferenceStatus.Unsupported => "REFERENCE_UNSUPPORTED",
                        _ => "REFERENCE_FAILURE"
                    };

                    throw new InvalidOperationException($"{category}: {resolution.DiagnosticCode}: {resolution.Message}");
                }

                var targetId = NodeChildId(parentId, "geometry", resolution.Target!.TargetId);
                AddRelationship(relationships, new CompiledRelationship(
                    $"relationship:{nodeId}:references:{EncodeSegment(resolution.Target.TargetId)}", "references", nodeId, [targetId], new Dictionary<string, JsonElement>()));
            }
            children.Add(nodeId);
        }

        return children.OrderBy(x => x, StringComparer.Ordinal).ToArray();
    }

    private static string AddOccurrence(
        AssemblySemantic containingAssembly,
        AssemblyOccurrenceSemantic occurrence,
        string parentId,
        SemanticApplication application,
        IDictionary<string, CompiledNode> nodes,
        IDictionary<string, CompiledRelationship> relationships)
    {
        var nodeId = parentId.StartsWith("definition:assembly:", StringComparison.Ordinal)
            ? OccurrenceId(containingAssembly.Id, occurrence.Id)
            : NodeChildId(parentId, "occurrence", occurrence.Id);

        var metadata = Metadata(occurrence.Metadata).ToDictionary(x => x.Key, x => x.Value, StringComparer.Ordinal);
        metadata["definitionId"] = JsonSerializer.SerializeToElement(DefinitionId(KindTitle(occurrence.DefinitionKind), occurrence.DefinitionId));
        metadata["definitionKind"] = JsonSerializer.SerializeToElement(occurrence.DefinitionKind);
        metadata["quantity"] = JsonSerializer.SerializeToElement(occurrence.Quantity);
        metadata["configuration"] = JsonSerializer.SerializeToElement(occurrence.ConfigurationName);
        metadata["bomStructure"] = JsonSerializer.SerializeToElement(occurrence.BomStructure);
        metadata["transform"] = JsonSerializer.SerializeToElement(occurrence.Transform.Matrix);

        var childIds = new List<string>();
        if (occurrence.DefinitionKind == "assembly")
        {
            var definition = application.Assemblies.Single(x => x.Id == occurrence.DefinitionId);
            foreach (var nested in definition.Occurrences.OrderBy(x => x.Id, StringComparer.Ordinal))
                childIds.Add(AddOccurrence(definition, nested, nodeId, application, nodes, relationships));
        }
        else if (occurrence.DefinitionKind == "part")
        {
            var definition = application.Parts.Single(x => x.Id == occurrence.DefinitionId);
            childIds.AddRange(AddPartChildren(application, definition, nodeId, nodes, relationships));
        }

        AddNode(nodes, new CompiledNode(
            nodeId,
            occurrence.Name,
            "ComponentInstance",
            parentId,
            childIds.OrderBy(x => x, StringComparer.Ordinal).ToArray(),
            new SortedDictionary<string, JsonElement>(metadata, StringComparer.Ordinal),
            [$"relationship:{nodeId}:instantiates"],
            [],
            new CompiledCapabilities(occurrence.Visible && !occurrence.Suppressed, true, true, true),
            null,
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["visible"] = occurrence.Visible.ToString().ToLowerInvariant(),
                ["suppressed"] = occurrence.Suppressed.ToString().ToLowerInvariant(),
                ["grounded"] = occurrence.Grounded.ToString().ToLowerInvariant(),
                ["flexible"] = occurrence.Flexible.ToString().ToLowerInvariant()
            }));

        var target = DefinitionId(KindTitle(occurrence.DefinitionKind), occurrence.DefinitionId);
        AddRelationship(relationships, new CompiledRelationship(
            $"relationship:{nodeId}:instantiates", "instantiates", nodeId, [target], new Dictionary<string, JsonElement>()));
        return nodeId;
    }

    private static void AddNode(IDictionary<string, CompiledNode> nodes, CompiledNode node)
    {
        if (!nodes.TryAdd(node.Id, node))
            throw new InvalidOperationException($"Compiled node ID '{node.Id}' is duplicated.");
    }

    private static void AddRelationship(IDictionary<string, CompiledRelationship> relationships, CompiledRelationship relationship)
    {
        if (!relationships.TryAdd(relationship.Id, relationship))
            throw new InvalidOperationException($"Compiled relationship ID '{relationship.Id}' is duplicated.");
    }

    private static void ValidateGraph(
        IReadOnlyDictionary<string, CompiledNode> nodes,
        IReadOnlyDictionary<string, CompiledRelationship> relationships,
        IReadOnlyList<string> roots)
    {
        foreach (var root in roots)
            if (!nodes.ContainsKey(root))
                throw new InvalidOperationException($"Compiled root '{root}' does not resolve.");

        foreach (var node in nodes.Values)
        {
            if (node.ParentId is not null && !nodes.ContainsKey(node.ParentId))
                throw new InvalidOperationException($"Compiled node '{node.Id}' references missing parent '{node.ParentId}'.");
            foreach (var child in node.ChildIds)
            {
                if (!nodes.ContainsKey(child))
                    throw new InvalidOperationException($"Compiled node '{node.Id}' references missing child '{child}'.");
                if (!string.Equals(nodes[child].ParentId, node.Id, StringComparison.Ordinal))
                    throw new InvalidOperationException($"Compiled child '{child}' does not point back to parent '{node.Id}'.");
            }
        }

        foreach (var relationship in relationships.Values)
        {
            if (!nodes.ContainsKey(relationship.SourceId))
                throw new InvalidOperationException($"Relationship '{relationship.Id}' source does not resolve.");
            foreach (var target in relationship.TargetIds)
                if (!nodes.ContainsKey(target))
                    throw new InvalidOperationException($"Relationship '{relationship.Id}' target '{target}' does not resolve.");
        }
    }

    private static IReadOnlyDictionary<string, JsonElement> Metadata(CadMetadata metadata) => Metadata((IReadOnlyDictionary<string, string>)metadata);

    private static IReadOnlyDictionary<string, JsonElement> Metadata(IReadOnlyDictionary<string, string> values) =>
        new SortedDictionary<string, JsonElement>(values.ToDictionary(x => x.Key, x => JsonSerializer.SerializeToElement(x.Value), StringComparer.Ordinal), StringComparer.Ordinal);

    private static string DefinitionId(string kind, string id) => $"definition:{kind.ToLowerInvariant()}:{EncodeSegment(id)}";
    private static string OccurrenceId(string assemblyId, string occurrenceId) => $"occurrence:{EncodeSegment(assemblyId)}/{EncodeSegment(occurrenceId)}";
    private static string NodeChildId(string parentId, string kind, string id) => $"{parentId}/{kind}:{EncodeSegment(id)}";
    private static string EncodeSegment(string value) => Uri.EscapeDataString(value);
    private static string KindTitle(string kind) => kind.Equals("assembly", StringComparison.Ordinal) ? "Assembly" : "Part";
}
