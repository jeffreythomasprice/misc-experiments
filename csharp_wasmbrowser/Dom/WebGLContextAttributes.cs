using System;
using System.Collections.Generic;
using System.Runtime.InteropServices.JavaScript;
using System.Text.Json;
using System.Text.Json.Serialization;
using Experiments.Dom;

public class WebGLContextAttributes
{
	[JsonConverter(typeof(PowerPreferenceTypeJsonConverter))]
	public enum PowerPreferenceType
	{
		Default,
		HighPerformance,
		LowPower,
	}

	// TODO make me generic
	public class PowerPreferenceTypeJsonConverter : JsonConverter<PowerPreferenceType>
	{
		private static readonly Dictionary<string, PowerPreferenceType> stringToEnum = new();
		private static readonly Dictionary<PowerPreferenceType, string> enumToString = new();

		static PowerPreferenceTypeJsonConverter()
		{
			foreach (var (stringValue, enumValue) in new[]
			{
				("default", PowerPreferenceType.Default),
				("high-performance", PowerPreferenceType.HighPerformance),
				("low-power", PowerPreferenceType.LowPower),
			})
			{
				stringToEnum[stringValue] = enumValue;
				enumToString[enumValue] = stringValue;
			}
		}

		public override PowerPreferenceType Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
		{
			var stringValue = reader.GetString();
			if (stringValue == null)
			{
				throw new NullReferenceException();
			}
			return stringToEnum[stringValue];
		}

		public override void Write(Utf8JsonWriter writer, PowerPreferenceType value, JsonSerializerOptions options)
		{
			writer.WriteStringValue(enumToString[value]);
		}
	}

	[JsonPropertyName("powerPreference")]
	[JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
	public PowerPreferenceType? PowerPreference { get; set; }

	// TODO rest of the props

	private string ToString(PowerPreferenceType value)
	{
		switch (value)
		{
			case PowerPreferenceType.Default:
				return "default";
			case PowerPreferenceType.HighPerformance:
				return "high-performance";
			case PowerPreferenceType.LowPower:
				return "low-power";
			default:
				throw new ArgumentException($"unhandled {value}");
		}
	}
}