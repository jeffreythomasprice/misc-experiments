using System.Numerics;
using System.Runtime.InteropServices.Swift;

namespace Robowar;

public record struct Vector2<T>(T X, T Y) where T :
	IUnaryPlusOperators<T, T>,
	IUnaryNegationOperators<T, T>,
	IAdditionOperators<T, T, T>,
	ISubtractionOperators<T, T, T>,
	IMultiplyOperators<T, T, T>,
	IDivisionOperators<T, T, T>
{
	public static Vector2<T> Zero => new Vector2<T>(default!, default!);
	public static Vector2<T> operator +(Vector2<T> a) => new(+a.X, +a.Y);
	public static Vector2<T> operator -(Vector2<T> a) => new(-a.X, -a.Y);
	public static Vector2<T> operator +(Vector2<T> a, Vector2<T> b) => new(a.X + b.X, a.Y + b.Y);
	public static Vector2<T> operator -(Vector2<T> a, Vector2<T> b) => new(a.X - b.X, a.Y - b.Y);
	public static Vector2<T> operator *(Vector2<T> a, T b) => new(a.X * b, a.Y * b);
	public static Vector2<T> operator /(Vector2<T> a, T b) => new(a.X / b, a.Y / b);
}
