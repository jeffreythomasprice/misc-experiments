using BlazorExperiments.Lib.Math;

namespace BlazorExperiments.Lib.StateMachine;

public record MouseMoveEvent(Vector2<int> Position, Vector2<int> Movement);