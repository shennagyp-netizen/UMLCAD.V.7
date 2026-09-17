using System.Diagnostics;
using System.Net.Http;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Framework;
using UMLCAD.Kernel.Client;

var e2e = args.Any(x => string.Equals(x, "--e2e", StringComparison.Ordinal));
var root = FindRepositoryRoot();
Console.WriteLine($"[UMLCAD] Repository root: {root}");

var kernelManifest = Path.Combine(root, "kernel_rust", "Cargo.toml");
var viewerDirectory = Path.Combine(root, "viewer");
if (!File.Exists(kernelManifest))
    throw new FileNotFoundException($"Rust kernel manifest was not found: {kernelManifest}");
if (!Directory.Exists(viewerDirectory))
    throw new DirectoryNotFoundException($"Viewer directory was not found: {viewerDirectory}");

var processDirectory = Path.GetTempPath();
Console.WriteLine($"[UMLCAD] Process working directory: {processDirectory}");
if (e2e)
    Console.WriteLine("[UMLCAD] Non-interactive E2E mode enabled; viewer startup will be skipped.");

var kernel = StartProcess(processDirectory, "cargo", $"run --release --bin kernel_host --manifest-path {Quote(kernelManifest)}");
var viewer = (Process?)null;
var runtimeDirectory = Path.Combine(viewerDirectory, "public", "__umlcad");
var runtimePackage = Path.Combine(runtimeDirectory, "demo.json");
var runtimeBuildPackage = Path.Combine(runtimeDirectory, "demo-build.json");

try
{
    Console.WriteLine("[UMLCAD] Starting Rust kernel...");
    await WaitForHttpAsync("http://127.0.0.1:8080/health", TimeSpan.FromMinutes(3));

    using var app = CadApplication.CreateBuilder()
        .ConfigureDemo()
        .Build();

    var package = app.CreateBuildPackage();
    Console.WriteLine($"[UMLCAD] Built demo '{app.Semantic.Id}' ({app.Semantic.BuildIdentity[..12]}...).");
    Console.WriteLine("[UMLCAD] Sending build package to Rust kernel for validation/evaluation...");

    using var httpClient = new HttpClient { BaseAddress = new Uri("http://127.0.0.1:8080/"), Timeout = TimeSpan.FromMinutes(2) };
    var service = new RustKernelService(
        httpClient,
        Options.Create(new RustKernelOptions
        {
            BaseAddress = new Uri("http://127.0.0.1:8080/"),
            RequestTimeout = TimeSpan.FromMinutes(2),
            MaxResponseBytes = 256L * 1024 * 1024
        }));

    var result = await service.EvaluateAsync(package);
    if (!result.Succeeded || result.CompiledModel is null)
    {
        foreach (var diagnostic in result.Diagnostics)
            Console.Error.WriteLine($"[{diagnostic.Severity}] {diagnostic.Code}: {diagnostic.Message}");
        throw new InvalidOperationException("Rust kernel rejected the demo build.");
    }

    var compiled = result.CompiledModel;
    Directory.CreateDirectory(runtimeDirectory);
    await File.WriteAllTextAsync(runtimePackage, JsonSerializer.Serialize(compiled, new JsonSerializerOptions(JsonSerializerDefaults.Web)));
    await File.WriteAllTextAsync(runtimeBuildPackage, JsonSerializer.Serialize(package, new JsonSerializerOptions(JsonSerializerDefaults.Web)));
    Console.WriteLine($"[UMLCAD] Kernel accepted build. Roots: {string.Join(", ", compiled.Manifest.RootNodeIds)}");

    if (e2e)
    {
        var manifest = compiled.Manifest;
        var rootNode = manifest.Nodes.Single(x => x.Id == manifest.RootNodeIds.Single(x => x == "definition:assembly:demo"));
        if (rootNode.Name != "Bench Vise Demo" || rootNode.Kind != "Assembly")
            throw new InvalidOperationException("Demo E2E root assembly did not survive kernel compilation.");
        foreach (var expectedName in new[] { "Body Assembly", "Screw Assembly", "Handle Assembly", "Sliding Jaw" })
            if (!manifest.Nodes.Any(x => x.Name == expectedName))
                throw new InvalidOperationException($"Demo E2E compiled model is missing '{expectedName}'.");
        Console.WriteLine($"[UMLCAD] E2E PASS: {manifest.Nodes.Count} nodes, {manifest.Relationships.Count} relationships, {manifest.RootNodeIds.Count} root.");
    }
    else
    {
        Console.WriteLine("[UMLCAD] Preparing browser viewer...");
        var nodeModules = Path.Combine(viewerDirectory, "node_modules");
        if (!Directory.Exists(nodeModules))
            RunAndRequireSuccess(processDirectory, "npm", $"install --no-audit --no-fund --prefix {Quote(viewerDirectory)}");

        viewer = StartProcess(processDirectory, "npm", $"run dev --prefix {Quote(viewerDirectory)} -- --host 127.0.0.1");
        await WaitForHttpAsync("http://127.0.0.1:4173/", TimeSpan.FromMinutes(1));

        var url = "http://127.0.0.1:4173/?package=/__umlcad/demo.json&build=/__umlcad/demo-build.json";
        Console.WriteLine($"[UMLCAD] Opening validated demo in browser: {url}");
        OpenBrowser(url);
        Console.WriteLine("[UMLCAD] Demo is running. Press Ctrl+C to stop the kernel and viewer.");

        using var stopped = new ManualResetEventSlim(false);
        Console.CancelKeyPress += (_, eventArgs) =>
        {
            eventArgs.Cancel = true;
            stopped.Set();
        };

        stopped.Wait();
    }
}
finally
{
    TryStop(viewer);
    TryStop(kernel);
    try
    {
        if (File.Exists(runtimePackage)) File.Delete(runtimePackage);
        if (File.Exists(runtimeBuildPackage)) File.Delete(runtimeBuildPackage);
        if (Directory.Exists(runtimeDirectory) && !Directory.EnumerateFileSystemEntries(runtimeDirectory).Any())
            Directory.Delete(runtimeDirectory);
    }
    catch { }
}

static string FindRepositoryRoot()
{
    var explicitRoot = Environment.GetEnvironmentVariable("UMLCAD_REPOSITORY_ROOT");
    if (!string.IsNullOrWhiteSpace(explicitRoot))
    {
        var full = Path.GetFullPath(explicitRoot);
        if (IsRepositoryRoot(full))
            return full;
        throw new DirectoryNotFoundException($"UMLCAD_REPOSITORY_ROOT does not contain kernel_rust/Cargo.toml and viewer/package.json: {full}");
    }

    foreach (var start in new[] { Directory.GetCurrentDirectory(), AppContext.BaseDirectory })
    {
        var directory = new DirectoryInfo(start);
        while (directory is not null)
        {
            if (IsRepositoryRoot(directory.FullName))
                return directory.FullName;
            directory = directory.Parent;
        }
    }

    throw new DirectoryNotFoundException(
        $"Could not locate the UMLCAD.V.5 repository root from '{Directory.GetCurrentDirectory()}' or '{AppContext.BaseDirectory}'. " +
        "Run from the repository or set UMLCAD_REPOSITORY_ROOT to the repository path.");
}

static bool IsRepositoryRoot(string directory) =>
    File.Exists(Path.Combine(directory, "kernel_rust", "Cargo.toml")) &&
    File.Exists(Path.Combine(directory, "viewer", "package.json"));

static Process StartProcess(string workingDirectory, string executable, string arguments)
{
    var command = OperatingSystem.IsWindows() && executable is "npm" or "cargo" ? executable + ".cmd" : executable;
    Console.WriteLine($"[UMLCAD] Starting: {command} {arguments}");
    Console.WriteLine($"[UMLCAD] Child process working directory: {workingDirectory}");

    var process = Process.Start(new ProcessStartInfo
    {
        FileName = command,
        Arguments = arguments,
        WorkingDirectory = workingDirectory,
        UseShellExecute = false,
        RedirectStandardOutput = false,
        RedirectStandardError = false,
        CreateNoWindow = false
    });
    return process ?? throw new InvalidOperationException($"Unable to start {executable}.");
}

static void RunAndRequireSuccess(string workingDirectory, string executable, string arguments)
{
    using var process = StartProcess(workingDirectory, executable, arguments);
    process.WaitForExit();
    if (process.ExitCode != 0)
        throw new InvalidOperationException($"{executable} {arguments} failed with exit code {process.ExitCode}.");
}

static string Quote(string value) => $"\"{value.Replace("\\", "\\\\").Replace("\"", "\\\"")}\"";

static async Task WaitForHttpAsync(string url, TimeSpan timeout)
{
    using var client = new HttpClient { Timeout = TimeSpan.FromSeconds(2) };
    var deadline = DateTime.UtcNow + timeout;
    while (DateTime.UtcNow < deadline)
    {
        try
        {
            using var response = await client.GetAsync(url);
            if (response.IsSuccessStatusCode)
                return;
        }
        catch (HttpRequestException) { }
        catch (TaskCanceledException) { }
        await Task.Delay(250);
    }

    throw new TimeoutException($"Timed out waiting for {url}.");
}

static void OpenBrowser(string url)
{
    if (OperatingSystem.IsMacOS())
    {
        Process.Start(new ProcessStartInfo
        {
            FileName = "open",
            Arguments = Quote(url),
            UseShellExecute = false,
            CreateNoWindow = true
        });
        return;
    }

    Process.Start(new ProcessStartInfo
    {
        FileName = url,
        UseShellExecute = true
    });
}

static void TryStop(Process? process)
{
    if (process is null) return;
    try
    {
        if (!process.HasExited)
            process.Kill(entireProcessTree: true);
    }
    catch { }
    finally
    {
        process.Dispose();
    }
}
