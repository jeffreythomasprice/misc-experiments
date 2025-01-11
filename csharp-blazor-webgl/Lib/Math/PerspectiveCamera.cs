using System.Drawing;
using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public class PerspectiveCamera<T> where T : INumber<T>, IRootFunctions<T>, ITrigonometricFunctions<T>
{
    private Size windowSize;
    private Radians<T> verticalFieldOfView;

    private readonly Vector3<T> defaultUp;
    private readonly Vector3<T> defaultForward;

    private Vector3<T> position;

    private Radians<T> angleRight;
    private Radians<T> angleUp;

    // TODO JEFF this is all janked up, double check all math

    public PerspectiveCamera(Size windowSize, Radians<T> verticalFieldOfView, Vector3<T> position, Vector3<T> target, Vector3<T> defaultUp)
    {
        // TODO check invalid args

        this.windowSize = windowSize;
        this.verticalFieldOfView = verticalFieldOfView;

        this.defaultUp = defaultUp.Normalized();

        this.position = position;

        var forward = (target - position).Normalized();
        if (T.Abs(forward.X) > T.Abs(forward.Y) && T.Abs(forward.X) > T.Abs(forward.Z))
        {
            if (T.Sign(forward.X) > 0)
            {
                defaultForward = new(T.One, T.Zero, T.Zero);
            }
            else
            {
                defaultForward = new(-T.One, T.Zero, T.Zero);
            }
        }
        else if (T.Abs(forward.Y) > T.Abs(forward.Z))
        {
            if (T.Sign(forward.Y) > 0)
            {
                defaultForward = new(T.Zero, T.One, T.Zero);
            }
            else
            {
                defaultForward = new(T.Zero, -T.One, T.Zero);
            }
        }
        else
        {
            if (T.Sign(forward.Z) > 0)
            {
                defaultForward = new(T.Zero, T.Zero, T.One);
            }
            else
            {
                defaultForward = new(T.Zero, T.Zero, -T.One);
            }
        }

        var defaultRight = defaultUp.CrossProduct(defaultForward);

        var q = new Plane<T>(defaultUp, position).ClosestPointTo(target);
        var u = q - position;
        var angleRight = defaultForward.AngleBetween(u);
        Radians<T> angleRightFixed;
        if (T.Sign(defaultRight.DotProduct(forward)) > 0)
        {
            angleRightFixed = angleRight;
        }
        else
        {
            angleRightFixed = new Radians<T>(T.Pi * T.CreateChecked(2)) - angleRight;
        }
        this.angleRight = FixAngleRight(angleRightFixed);

        var g = target - position;
        var angleUp = g.AngleBetween(u);
        Radians<T> angleUpFixed;
        if (T.Sign(defaultUp.DotProduct(forward)) > 0)
        {
            angleUpFixed = angleUp;
        }
        else
        {
            angleUpFixed = -angleUp;
        }
        this.angleUp = FixAngleUp(angleUpFixed);
    }

    public Size WindowSize
    {
        get => windowSize;
        set
        {
            windowSize = value;
            // TODO make dirty
        }
    }

    public Radians<T> VerticalFieldOfView
    {
        get => verticalFieldOfView;
        set
        {
            verticalFieldOfView = value;
            // TODO make dirty
        }
    }

    public Radians<T> AngleRight
    {
        get => angleRight;
        // TODO make dirty
        set => angleRight = FixAngleRight(value);
    }

    public Radians<T> AngleUp
    {
        get => angleUp;
        // TODO make dirty
        set => angleUp = FixAngleUp(value);
    }

    public Vector3<T> Position
    {
        get => position;
        // TODO make dirty
        set => position = value;
    }

    public void Turn(Vector2<T> mouseMovement)
    {
        // TODO put constants somewhere
        var v = mouseMovement / T.CreateChecked(700);
        angleRight += new Degrees<T>(T.CreateChecked(45)).Radians * new Radians<T>(v.X);
        angleUp -= new Degrees<T>(T.CreateChecked(45)).Radians * new Radians<T>(v.Y);
    }

    public void Move(T forward, T strafe, T up)
    {
        position += Forward * forward + RightRightAngleOnly * strafe + defaultUp * up;
    }

    // TODO cache
    public Matrix4<T> ProjectionMatrix => Matrix4<T>.CreatePerspective(
        verticalFieldOfView,
        T.CreateChecked(windowSize.Width),
        T.CreateChecked(windowSize.Height),
        // TODO near and far should be configurable
        T.CreateChecked(0.01),
        T.CreateChecked(1000.0)
    );

    // TODO cache
    public Matrix4<T> ModelViewMatrix => Matrix4<T>.CreateLookAt(position, position + Forward, defaultUp);

    // TODO cache
    private Vector3<T> Forward => Matrix4<T>.CreateRotation(RightRightAngleOnly, angleUp).ApplyToVector(ForwardRightAngleOnly);

    // TODO cache
    private Vector3<T> RightRightAngleOnly => defaultUp.CrossProduct(ForwardRightAngleOnly);

    // TODO cache
    private Vector3<T> ForwardRightAngleOnly => Matrix4<T>.CreateRotation(defaultUp, angleRight).ApplyToVector(defaultForward);

    private static Radians<T> FixAngleRight(Radians<T> value)
    {
        // TODO put constants somewhere
        var pi2 = new Radians<T>(T.Pi * T.CreateChecked(2));
        var x = value % pi2;
        if (x < new Radians<T>(T.Zero))
        {
            return x + pi2;
        }
        else
        {
            return x;
        }
    }

    private static Radians<T> FixAngleUp(Radians<T> value)
    {
        // TODO put constants somewhere
        var limit = new Radians<T>(T.Pi * T.CreateChecked(0.49));
        return Radians<T>.Clamp(value, -limit, limit);
    }
}