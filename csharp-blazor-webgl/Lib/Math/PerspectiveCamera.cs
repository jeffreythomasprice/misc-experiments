using System.Drawing;
using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public class PerspectiveCamera<T> where T : INumber<T>, IRootFunctions<T>, ITrigonometricFunctions<T>
{
    private static Radians<T> TwoPi = new(T.Pi * T.CreateChecked(2));
    private static readonly Radians<T> UpAngleLimit = new(T.Pi * T.CreateChecked(0.49));
    private static readonly T TurnSpeed = T.CreateChecked(1) / T.CreateChecked(700);

    private Size windowSize;
    private Radians<T> verticalFieldOfView;
    private T nearClipPlane;
    private T farClipPlane;

    private readonly Vector3<T> defaultUp;
    private readonly Vector3<T> defaultForward;

    private Vector3<T> position;

    private Radians<T> angleRight;
    private Radians<T> angleUp;

    private Matrix4<T>? projectionMatrix;
    private Matrix4<T>? modelViewMatrix;
    private Vector3<T>? forward;
    private Vector3<T>? rightRightAngleOnly;
    private Vector3<T>? forwardRightAngleOnly;

    public PerspectiveCamera(Size windowSize, Radians<T> verticalFieldOfView, T nearClipPlane, T farClipPlane, Vector3<T> position, Vector3<T> target, Vector3<T> defaultUp)
    {
        if (position == target)
        {
            throw new ArgumentException("camera position and target can't be the same point");
        }
        if (defaultUp == new Vector3<T>(T.Zero, T.Zero, T.Zero))
        {
            throw new ArgumentException("up vector can't be (0,0,0)");
        }

        this.windowSize = windowSize;
        this.verticalFieldOfView = verticalFieldOfView;
        this.nearClipPlane = nearClipPlane;
        this.farClipPlane = farClipPlane;

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
            angleRightFixed = TwoPi - angleRight;
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
            projectionMatrix = null;
        }
    }

    public Radians<T> VerticalFieldOfView
    {
        get => verticalFieldOfView;
        set
        {
            verticalFieldOfView = value;
            projectionMatrix = null;
        }
    }

    public T NearClipPlane
    {
        get => nearClipPlane;
        set
        {
            nearClipPlane = value;
            projectionMatrix = null;
        }
    }

    public T FarClipPlane
    {
        get => farClipPlane;
        set
        {
            farClipPlane = value;
            projectionMatrix = null;
        }
    }

    public Radians<T> AngleRight
    {
        get => angleRight;
        set
        {
            angleRight = FixAngleRight(value);
            modelViewMatrix = null;
            forward = null;
            rightRightAngleOnly = null;
            forwardRightAngleOnly = null;
        }
    }

    public Radians<T> AngleUp
    {
        get => angleUp;
        set
        {
            angleUp = FixAngleUp(value);
            modelViewMatrix = null;
            forward = null;
            rightRightAngleOnly = null;
            forwardRightAngleOnly = null;
        }
    }

    public Vector3<T> Position
    {
        get => position;
        set
        {
            position = value;
            modelViewMatrix = null;
        }
    }

    public void Turn(Vector2<T> mouseMovement)
    {
        var v = mouseMovement * TurnSpeed;
        AngleRight += new Degrees<T>(T.CreateChecked(45)).Radians * new Radians<T>(v.X);
        AngleUp -= new Degrees<T>(T.CreateChecked(45)).Radians * new Radians<T>(v.Y);
    }

    public void Move(T forward, T strafe, T up)
    {
        Position += Forward * forward - RightRightAngleOnly * strafe + defaultUp * up;
    }

    public Matrix4<T> ProjectionMatrix => projectionMatrix ??= Matrix4<T>.CreatePerspective(
        verticalFieldOfView,
        T.CreateChecked(windowSize.Width),
        T.CreateChecked(windowSize.Height),
        nearClipPlane,
        farClipPlane
    );

    public Matrix4<T> ModelViewMatrix => modelViewMatrix ??= Matrix4<T>.CreateLookAt(position, position + Forward, defaultUp);

    private Vector3<T> Forward => forward ??= Matrix4<T>.CreateRotation(RightRightAngleOnly, AngleUp).ApplyToVector(ForwardRightAngleOnly);

    private Vector3<T> RightRightAngleOnly => rightRightAngleOnly ??= defaultUp.CrossProduct(ForwardRightAngleOnly);

    private Vector3<T> ForwardRightAngleOnly => forwardRightAngleOnly ??= Matrix4<T>.CreateRotation(defaultUp, AngleRight).ApplyToVector(defaultForward);

    private static Radians<T> FixAngleRight(Radians<T> value)
    {
        var x = value % TwoPi;
        if (x < new Radians<T>(T.Zero))
        {
            return x + TwoPi;
        }
        else
        {
            return x;
        }
    }

    private static Radians<T> FixAngleUp(Radians<T> value)
    {
        return Radians<T>.Clamp(value, -UpAngleLimit, UpAngleLimit);
    }
}