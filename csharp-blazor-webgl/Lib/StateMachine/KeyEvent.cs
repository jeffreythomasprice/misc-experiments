namespace BlazorExperiments.Lib.StateMachine;

public record KeyEvent(KeyboardKey Key, string RawKey, string RawCode)
{
    public KeyEvent(string key, string code) : this(KeyboardKeyExtensions.FromCode(code), key, code) { }
}