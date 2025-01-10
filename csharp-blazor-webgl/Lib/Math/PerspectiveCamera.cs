using System.Drawing;

namespace BlazorExperiments.Lib.Math;

public class PerspectiveCamera
{
    private Size windowSize;
    private Radians<float> verticalFieldOfView;

    private readonly Vector3<float> defaultUp;
    private readonly Vector3<float> defaultForward;

    private Vector3<float> position;

    private Radians<float> angleRight;
    private Radians<float> angleUp;

    private Matrix4<float>? projectionMatrix;
    private Matrix4<float>? modelViewMatrix;

    public PerspectiveCamera(Size windowSize, Radians<float> verticalFieldOfView, Vector3<float> position, Vector3<float> target, Vector3<float> defaultUp)
    {
        // TODO check invalid args

        this.windowSize = windowSize;
        this.verticalFieldOfView = verticalFieldOfView;

        this.defaultUp = defaultUp.Normalized();
        Console.WriteLine($"TODO defaultUp = {defaultUp}");

        this.position = position;
        Console.WriteLine($"TODO position = {position}");
        Console.WriteLine($"TODO target = {target}");

        var forward = (target - position).Normalized();
        Console.WriteLine($"TODO forward = {forward}");
        if (MathF.Abs(forward.X) > MathF.Abs(forward.Y) && MathF.Abs(forward.X) > MathF.Abs(forward.Z))
        {
            if (forward.X > 0)
            {
                defaultForward = new(1, 0, 0);
            }
            else
            {
                defaultForward = new(-1, 0, 0);
            }
        }
        else if (MathF.Abs(forward.Y) > MathF.Abs(forward.Z))
        {
            if (forward.Y > 0)
            {
                defaultForward = new(0, 1, 0);
            }
            else
            {
                defaultForward = new(0, -1, 0);
            }
        }
        else
        {
            if (forward.Z > 0)
            {
                defaultForward = new(0, 0, 1);
            }
            else
            {
                defaultForward = new(0, 0, -1);
            }
        }
        Console.WriteLine($"TODO defaultForward = {defaultForward}");

        var defaultRight = defaultUp.CrossProduct(defaultForward);
        Console.WriteLine($"TODO defaultRight = {defaultRight}");

        var q = new Plane<float>(defaultUp, position).ClosestPointTo(target);
        var u = q - position;
        var angleRight = defaultForward.AngleBetween(u);
        Radians<float> angleRightFixed;
        if (defaultRight.DotProduct(forward) > 0)
        {
            angleRightFixed = angleRight;
        }
        else
        {
            angleRightFixed = new Radians<float>(float.Pi * 2.0f) - angleRight;
        }
        this.angleRight = FixAngleRight(angleRightFixed);

        var g = target - position;
        var angleUp = g.AngleBetween(u);
        Radians<float> angleUpFixed;
        if (defaultUp.DotProduct(forward) > 0)
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
            modelViewMatrix = null;
        }
    }

    public Radians<float> VerticalFieldOfView
    {
        get => verticalFieldOfView;
        set
        {
            verticalFieldOfView = value;
            projectionMatrix = null;
            modelViewMatrix = null;
        }
    }

    /*
    TODO rest of camera 

    var angleRight: Radians<T> {
        get { self._angleRight }
        set {
            self._angleRight = LookAtCamera.fixAngleRight(value: newValue)
        }
    }

    var angleUp: Radians<T> {
        get { self._angleUp }
        set {
            self._angleUp = LookAtCamera.fixAngleUp(value: newValue)
        }
    }

    var position: Vector3<T> {
        get { self._position }
        set { self._position = newValue }
    }

    func turn(mouseMovement: Vector2<T>) {
        // TODO put constants somewhere
        let v = mouseMovement / 700
        angleRight = angleRight + Degrees(45).radians * Radians(v.x)
        angleUp = angleUp + Degrees(45).radians * Radians(v.y)
    }

    func move(forward: T, strafe: T, up: T) {
        position =
            position
            + self.forward * forward
            + rightRightAngleOnly * strafe
            + defaultUp * up
    }

    var transformMatrix: Matrix4<T> {
        // TODO cache
        return Matrix4.lookAt(
            position: position,
            target: position + forward,
            up: defaultUp
        )
    }

    private var forward: Vector3<T> {
        // TODO cache
        return Matrix4.rotation(axis: rightRightAngleOnly, angle: angleUp).applyTo(vector: forwardRightAngleOnly)
    }

    private var rightRightAngleOnly: Vector3<T> {
        // TODO cache
        Vector3.cross(defaultUp, forwardRightAngleOnly)
    }

    private var forwardRightAngleOnly: Vector3<T> {
        // TODO cache
        Matrix4.rotation(axis: defaultUp, angle: angleRight).applyTo(vector: defaultForward)
    }
    */

    private static Radians<float> FixAngleRight(Radians<float> value)
    {
        // TODO put constants somewhere
        var pi2 = new Radians<float>(float.Pi * 2.0f);
        var x = value % pi2;
        if (x < new Radians<float>(0))
        {
            return x + pi2;
        }
        else
        {
            return x;
        }
    }

    private static Radians<float> FixAngleUp(Radians<float> value)
    {
        // TODO put constants somewhere
        var limit = new Radians<float>(float.Pi * 0.49f);
        return Radians<float>.Clamp(value, -limit, limit);
    }
}