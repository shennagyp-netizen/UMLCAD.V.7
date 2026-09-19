using UMLCAD.Framework;

using var application = UmlcadApplication.ConnectDefault();

Console.WriteLine("UMLCAD.Application host");
Console.WriteLine("Application Layer: app/framework/libraries");
Console.WriteLine($"Application facade: {application.GetType().Name}");
