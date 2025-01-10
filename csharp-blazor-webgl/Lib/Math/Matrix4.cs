using System.Numerics;

namespace BlazorExperiments.Lib.Math;

public struct Matrix4<T> :
    IMultiplyOperators<Matrix4<T>, Matrix4<T>, Matrix4<T>>
    where T : INumber<T>, IRootFunctions<T>, ITrigonometricFunctions<T>
{
    public T[] data;

    public static Matrix4<T> Identity
    {
        get
        {
            return new()
            {
                data = [
                    T.One, T.Zero, T.Zero, T.Zero,
                    T.Zero, T.One, T.Zero, T.Zero,
                    T.Zero, T.Zero, T.One, T.Zero,
                    T.Zero, T.Zero, T.Zero, T.One,
                ]
            };
        }
    }

    public static Matrix4<T> CreateTranslation(Vector3<T> v)
    {
        return new()
        {
            data = [
                T.One, T.Zero, T.Zero, T.Zero,
                T.Zero, T.One, T.Zero, T.Zero,
                T.Zero, T.Zero, T.One, T.Zero,
                v.X, v.Y, v.Z, T.One,
            ]
        };
    }

    public static Matrix4<T> CreateScale(Vector3<T> v)
    {
        return new()
        {
            data = [
                v.X, T.Zero, T.Zero, T.Zero,
                T.Zero, v.Y, T.Zero, T.Zero,
                T.Zero, T.Zero, v.Z, T.Zero,
                T.Zero, T.Zero, T.Zero, T.One,
            ]
        };
    }

    public static Matrix4<T> CreateRotation(Vector3<T> axis, Radians<T> angle)
    {
        var c = Radians<T>.Cos(angle);
        var s = Radians<T>.Sin(angle);
        axis = axis.Normalized();
        return new()
        {
            data = [
                axis.X * axis.X * (T.One - c) + c, axis.X * axis.Y * (T.One - c) - axis.Z * s, axis.X * axis.Z * (T.One - c) + axis.Y * s, T.Zero,
                axis.Y * axis.X * (T.One - c) + axis.Z * s, axis.Y * axis.Y * (T.One - c) + c, axis.Y * axis.Z * (T.One - c) - axis.X * s, T.Zero,
                axis.X * axis.Z * (T.One - c) - axis.Y * s, axis.Y * axis.Z * (T.One - c) + axis.X * s, axis.Z * axis.Z * (T.One - c) + c, T.Zero,
                T.Zero, T.Zero, T.Zero, T.One,
            ],
        };
    }

    public static Matrix4<T> CreateOrtho(T left, T right, T bottom, T top, T near, T far)
    {
        var two = T.One + T.One;
        return new()
        {
            data = [
                two / (right - left), T.Zero, T.Zero, -(right + left) / (right - left),
                T.Zero, two / (top - bottom), T.Zero, -(top + bottom) / (top - bottom),
                T.Zero, T.Zero, -two / (far - near), -(far + near) / (far - near),
                T.Zero, T.Zero, T.Zero, T.One,
            ]
        };
    }

    public static Matrix4<T> CreatePerspective(Radians<T> verticalFieldOfView, T width, T height, T near, T far)
    {
        var two = T.One + T.One;
        var f = T.One / T.Tan(verticalFieldOfView.Value / two);
        var aspect = width / height;
        return new()
        {
            data = [
                f / aspect, T.Zero, T.Zero, T.Zero,
                T.Zero, f, T.Zero, T.Zero,
                T.Zero, T.Zero, (far + near) / (near - far), two * far * near / (near - far),
                T.Zero, T.Zero, -T.One, T.Zero,
            ]
        };
    }

    public static Matrix4<T> CreateLookAt(Vector3<T> position, Vector3<T> target, Vector3<T> up)
    {
        var f = (target - position).Normalized();
        up = up.Normalized();
        var s = f.CrossProduct(up).Normalized();
        var u = s.CrossProduct(f).Normalized();
        return new Matrix4<T>()
        {
            data = [
                s.X, u.X, -f.X, T.Zero,
                s.Y, u.Y, -f.Y, T.Zero,
                s.Z, u.Z, -f.Z, T.Zero,
                T.Zero, T.Zero, T.Zero, T.One,
            ]
        }.Translate(-position);
    }

    public static Matrix4<T> operator *(Matrix4<T> left, Matrix4<T> right)
    {
        var result = new T[16];
        for (var i = 0; i < 4; i++)
        {
            for (var j = 0; j < 4; j++)
            {
                for (var k = 0; k < 4; k++)
                {
                    result[i * 4 + j] += left.data[i * 4 + k] * right.data[k * 4 + j];
                }
            }
        }
        return new() { data = result };
    }

    public ReadOnlySpan<T> Data => data;

    public Matrix4<T> Translate(Vector3<T> v)
    {
        return Matrix4<T>.CreateTranslation(v) * this;
    }

    public Matrix4<T> Scale(Vector3<T> v)
    {
        return Matrix4<T>.CreateScale(v) * this;
    }

    public Matrix4<T> Rotate(Vector3<T> axis, Radians<T> angle)
    {
        return Matrix4<T>.CreateRotation(axis, angle) * this;
    }

    public Vector3<T> ApplyToPoint(Vector3<T> point)
    {
        return new(
            data[0] * point.X + data[4] * point.Y + data[8] * point.Z + data[12],
            data[1] * point.X + data[5] * point.Y + data[9] * point.Z + data[13],
            data[2] * point.X + data[6] * point.Y + data[10] * point.Z + data[14]
        );
    }

    public Vector3<T> ApplyToVector(Vector3<T> vector)
    {
        return new(
            data[0] * vector.X + data[4] * vector.Y + data[8] * vector.Z,
            data[1] * vector.X + data[5] * vector.Y + data[9] * vector.Z,
            data[2] * vector.X + data[6] * vector.Y + data[10] * vector.Z
        );
    }
}
