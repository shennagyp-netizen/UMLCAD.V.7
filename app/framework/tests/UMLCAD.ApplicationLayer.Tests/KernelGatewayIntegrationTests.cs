using System.Net;
using System.Net.Sockets;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;
using UMLCAD.Kernel;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class KernelGatewayIntegrationTests
{
    [Fact]
    public async Task Typed_Cad_Evaluation_Travels_Through_Concrete_Kernel_Gateway()
    {
        await using var server = await LoopbackServer.StartAsync();

        using var kernel = UmlcadKernel.Connect(new UmlcadKernelOptions
        {
            BaseAddress = server.BaseAddress,
            BuildEvaluationPath = "v1/build/evaluate",
            RequestTimeout = TimeSpan.FromSeconds(10),
            MaximumResponseBytes = 1024 * 1024
        });

        var engine = new CadEvaluationEngine(kernel);
        var part = new CadPartDefinition(
            new CadId("part-integration"),
            "Integration Part",
            [
                new CadCircle(
                    new CadId("circle-integration"),
                    new CadPoint2D(2, 3),
                    1.5)
            ]);

        var result = await engine.EvaluateAsync(
            part,
            new CadBuildIdentity(
                "integration-test",
                "1.0.0",
                "typed-build-001"));

        Assert.True(result.Succeeded);
        Assert.NotNull(result.CompiledModel);
        Assert.Contains("circle-integration", server.RequestBody!);
        Assert.Contains(@"""kind"":""circle""", server.RequestBody!);
    }

    [Fact]
    public async Task KernelGateway_Uses_Real_Loopback_Transport_Through_Public_API()
    {
        await using var server = await LoopbackServer.StartAsync();

        using var kernel = UmlcadKernel.Connect(new UmlcadKernelOptions
        {
            BaseAddress = server.BaseAddress,
            BuildEvaluationPath = "v1/build/evaluate",
            RequestTimeout = TimeSpan.FromSeconds(10),
            MaximumResponseBytes = 1024 * 1024
        });

        using var semantic = System.Text.Json.JsonDocument.Parse("{\"parts\":[]}");
        var result = await kernel.EvaluateBuildAsync(
            new KernelBuildRequest(
                "integration-test",
                "1.0.0",
                "integration-build-001",
                semantic.RootElement.Clone()));

        Assert.True(result.Succeeded);
        Assert.NotNull(result.CompiledModel);
        Assert.Empty(result.Diagnostics);
        Assert.Equal("/v1/build/evaluate", server.RequestPath);
    }

    private sealed class LoopbackServer : IAsyncDisposable
    {
        private readonly TcpListener _listener;
        private readonly Task _serverTask;

        private LoopbackServer(TcpListener listener)
        {
            _listener = listener;
            BaseAddress = new Uri($"http://127.0.0.1:{((System.Net.IPEndPoint)listener.LocalEndpoint).Port}/");
            _serverTask = ServeAsync();
        }

        public Uri BaseAddress { get; }

        public string? RequestPath { get; private set; }

        public static Task<LoopbackServer> StartAsync()
        {
            var listener = new TcpListener(System.Net.IPAddress.Loopback, 0);
            listener.Start();
            return Task.FromResult(new LoopbackServer(listener));
        }

        private async Task ServeAsync()
        {
            using var client = await _listener.AcceptTcpClientAsync();
            await using var stream = client.GetStream();

            var request = await ReadRequestAsync(stream);
            var requestLine = request.Split("\r\n", 2)[0];
            var parts = requestLine.Split(' ', StringSplitOptions.RemoveEmptyEntries);

            if (parts.Length >= 2)
                RequestPath = parts[1];

            const string body = """
            {
              "succeeded": true,
              "compiledModel": {
                "schema": "uml-cad-compiled-model/1.1.0"
              },
              "diagnostics": []
            }
            """;

            var bytes = Encoding.UTF8.GetBytes(body);
            var response =
                $"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {bytes.Length}\r\nConnection: close\r\n\r\n";

            await stream.WriteAsync(Encoding.ASCII.GetBytes(response));
            await stream.WriteAsync(bytes);
        }

        private static async Task<string> ReadRequestAsync(NetworkStream stream)
        {
            var data = new List<byte>();
            var buffer = new byte[4096];

            while (true)
            {
                var read = await stream.ReadAsync(buffer);
                if (read == 0)
                    break;

                data.AddRange(buffer.AsSpan(0, read).ToArray());

                var text = Encoding.ASCII.GetString(data.ToArray());
                var headerEnd = text.IndexOf("\r\n\r\n", StringComparison.Ordinal);
                if (headerEnd >= 0)
                {
                    var headerLength = headerEnd + 4;
                    var contentLength = ParseContentLength(text[..headerEnd]);

                    if (data.Count >= headerLength + contentLength)
                        return text;
                }

                if (data.Count > 1024 * 1024)
                    throw new InvalidOperationException("Integration test request exceeded its safety limit.");
            }

            throw new EndOfStreamException("Loopback request ended before a complete HTTP request was received.");
        }

        private static int ParseContentLength(string headers)
        {
            foreach (var line in headers.Split("\r\n"))
            {
                if (!line.StartsWith("Content-Length:", StringComparison.OrdinalIgnoreCase))
                    continue;

                return int.Parse(line["Content-Length:".Length..].Trim());
            }

            return 0;
        }

        public async ValueTask DisposeAsync()
        {
            _listener.Stop();
            try
            {
                await _serverTask.WaitAsync(TimeSpan.FromSeconds(2));
            }
            catch (TimeoutException)
            {
            }
        }
    }
}
