using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;

namespace UMLCAD.Engineering.Tests;

public sealed class CoordinateFrameContractTests
{
    [Fact]
    public void DefaultFrameUsesCanonicalRightHandedBasis()
    {
        var frame = new CadFrame(
            new CadId("part"),
            CadFrameKind.Part,
            10d,
            20d,
            30d);

        frame.Validate();

        Assert.Equal(new CadVector3(1d, 0d, 0d), frame.XAxis);
        Assert.Equal(new CadVector3(0d, 1d, 0d), frame.YAxis);
        Assert.Equal(new CadVector3(0d, 0d, 1d), frame.ZAxis);
    }

    [Fact]
    public void OrientedFrameAcceptsAValidRightHandedBasis()
    {
        var frame = new CadFrame(
            new CadId("sketch"),
            CadFrameKind.Sketch,
            0d,
            0d,
            0d)
        {
            XAxis = new CadVector3(0d, 1d, 0d),
            YAxis = new CadVector3(-1d, 0d, 0d),
            ZAxis = new CadVector3(0d, 0d, 1d),
        };

        frame.Validate();
    }

    [Fact]
    public void NonOrthonormalFrameFailsClosed()
    {
        var frame = new CadFrame(
            new CadId("invalid"),
            CadFrameKind.Sketch,
            0d,
            0d,
            0d)
        {
            XAxis = new CadVector3(2d, 0d, 0d),
        };

        Assert.Throws<ArgumentException>(() => frame.Validate());
    }

    [Fact]
    public void LeftHandedFrameFailsClosed()
    {
        var frame = new CadFrame(
            new CadId("invalid"),
            CadFrameKind.Sketch,
            0d,
            0d,
            0d)
        {
            XAxis = new CadVector3(1d, 0d, 0d),
            YAxis = new CadVector3(0d, -1d, 0d),
            ZAxis = new CadVector3(0d, 0d, 1d),
        };

        Assert.Throws<ArgumentException>(() => frame.Validate());
    }

    [Fact]
    public void EvaluationIdentityChangesWhenFrameOrientationChanges()
    {
        var firstFrame = new CadFrame(
            new CadId("frame"),
            CadFrameKind.Sketch,
            0d,
            0d,
            0d);

        var secondFrame = firstFrame with
        {
            XAxis = new CadVector3(0d, 1d, 0d),
            YAxis = new CadVector3(-1d, 0d, 0d),
            ZAxis = new CadVector3(0d, 0d, 1d),
        };

        var first = new BoxFeatureSpecification(
            new CadId("feature"),
            firstFrame,
            2d,
            3d,
            4d);

        var second = first with { Frame = secondFrame };

        var firstIdentity = EvaluationIdentity.Compute(
            first,
            Array.Empty<ReferenceResolution>(),
            Array.Empty<CadFeatureEvaluationResult>(),
            "semantic-default",
            "A|default");

        var secondIdentity = EvaluationIdentity.Compute(
            second,
            Array.Empty<ReferenceResolution>(),
            Array.Empty<CadFeatureEvaluationResult>(),
            "semantic-default",
            "A|default");

        Assert.NotEqual(firstIdentity, secondIdentity);
    }
}


[Fact]
public void RotatedFrameTransformsPointsAndVectorsDeterministically()
{
    var frame = new CadFrame(
        new CadId("rotated"),
        CadFrameKind.Sketch,
        10d,
        20d,
        30d)
    {
        XAxis = new CadVector3(0d, 1d, 0d),
        YAxis = new CadVector3(-1d, 0d, 0d),
        ZAxis = new CadVector3(0d, 0d, 1d),
    };

    frame.Validate();

    var worldPoint = frame.ToWorldPoint(new CadVector3(2d, 3d, 4d));
    var worldVector = frame.ToWorldVector(new CadVector3(2d, 3d, 4d));

    Assert.Equal(new CadVector3(7d, 22d, 34d), worldPoint);
    Assert.Equal(new CadVector3(-3d, 2d, 4d), worldVector);
    Assert.Equal(new CadVector3(2d, 3d, 4d), frame.ToLocalPoint(worldPoint));
    Assert.Equal(new CadVector3(2d, 3d, 4d), frame.ToLocalVector(worldVector));
}
