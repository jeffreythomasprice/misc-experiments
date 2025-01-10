using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public struct Plane<T> where T : INumber<T>, IRootFunctions<T>
{
    public readonly Vector3<T> Normal;
    public readonly T D;

    public Plane(Vector3<T> normal, Vector3<T> point)
    {
        Normal = normal.Normalized();
        D = Normal.DotProduct(point);
    }

    public T SignedDistanceTo(Vector3<T> point)
    {
        return D - Normal.DotProduct(point);
    }

    public Vector3<T> ClosestPointTo(Vector3<T> point)
    {
        return Normal * SignedDistanceTo(point) + point;
    }
}
