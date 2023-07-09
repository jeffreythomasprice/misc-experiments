package com.jeffreythomasprice.experiments

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.plugins.statuspages.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import kotlinx.serialization.json.*
import net.pwall.json.*
import net.pwall.json.schema.JSONSchema
import net.pwall.json.schema.output.BasicErrorEntry
import org.slf4j.LoggerFactory
import java.io.File

class JSONSchemaValidator<T : Any>(
	val schema: JSONSchema
) {
	val logger = LoggerFactory.getLogger(this::class.java)!!

	companion object {
		fun <T : Any> fromResource(path: String) =
			getSchemaFromResource(path)?.let { JSONSchemaValidator<T>(it) }
	}
}

class JSONValidatorException(message: String, val errors: List<BasicErrorEntry>? = null) : Exception(message)

fun getResourceAsFile(path: String) =
	object {}.javaClass.getResource(path)?.let { File(it.toURI()) }

fun getSchemaFromResource(path: String) =
	getResourceAsFile(path)?.let { JSONSchema.parse(it) }

inline fun <reified T : Any> JSONSchemaValidator<T>.validate(input: String): T {
	val json = try {
		Json.parseToJsonElement(input)
	} catch (e: Exception) {
		throw JSONValidatorException("pre-validation deserialization failed for ${T::class}: ${e.localizedMessage}")
	}
	return validate(json)
}

inline fun <reified T : Any> JSONSchemaValidator<T>.validate(input: JsonElement): T {
	logger.trace("schema validation for input {}", input)
	val validationResult = try {
		schema.validateBasic(input.toJSONValue())
	} catch (e: Exception) {
		throw JSONValidatorException("schema validation failed for ${T::class}: ${e.localizedMessage}")
	}
	if (validationResult.valid) {
		val json = Json
		val result = json.decodeFromJsonElement<T>(input)
		logger.trace("produced result from json {}", result)
		return result
	} else {
		throw JSONValidatorException(
			"post-validation deserialization failed for ${T::class}",
			validationResult.errors
		)
	}
}

fun JsonElement.toJSONValue(): JSONValue? = when (this) {
	is JsonPrimitive -> this.toJSONValue()
	is JsonArray -> JSONArray(this.map { it.toJSONValue() })
	is JsonObject -> JSONObject(this.mapValues { it.value.toJSONValue() })
}

fun JsonPrimitive.toJSONValue(): JSONValue? {
	if (isString) {
		return JSONString(content)
	}
	if (this is JsonNull) {
		return null
	}
	this.booleanOrNull?.let {
		return JSONBoolean(it)
	}
	// don't play games with what kind of number data type best represents the input just re-parse
	return JSON.parse(content)
}

suspend inline fun <reified T : Any> ApplicationCall.receive(validator: JSONSchemaValidator<T>): T {
	return validator.validate(receive<String>())
}

suspend inline fun <reified T : Any> ApplicationCall.receive(schema: JSONSchema): T =
	receive(JSONSchemaValidator(schema))

fun StatusPagesConfig.jsonValidator() {
	val logger = LoggerFactory.getLogger(object {}.javaClass)
	exception<JSONValidatorException> { call, cause ->
		logger.error("json schema validation exception: ${cause.message ?: "missing message"}")
		val messages = mutableListOf<String>()
		cause.message?.let { messages.add(it) }
		cause.errors
			?.map { "${it.instanceLocation} ${it.error}" }
			?.let { messages.addAll(it) }
		call.respond(
			status = HttpStatusCode.BadRequest,
			message = ErrorResponse(messages)
		)
	}
}