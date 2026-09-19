using UMLCAD.Framework;
using UMLCAD.Kernel;

using var httpClient = new HttpClient
{
    BaseAddress = new Uri("http://127.0.0.1:8080/")
};

var application = new UmlcadApplication(new UmlcadKernel(httpClient));

Console.WriteLine("UMLCAD.Application host");
Console.WriteLine("Application Layer: app/framework/libraries");
Console.WriteLine($"Application facade: {application.GetType().Name}");
Console.WriteLine($"Kernel gateway: {typeof(UmlcadKernel).Assembly.GetName().Name}");
