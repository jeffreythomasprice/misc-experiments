package com.jeffreythomasprice.experiments

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import net.pwall.json.JSONException
import net.pwall.json.schema.JSONSchema
import org.http4k.core.*
import org.http4k.format.KotlinxSerialization.auto
import org.http4k.format.KotlinxSerialization.json
import org.http4k.format.httpBodyLens
import org.http4k.lens.BiDiBodyLens
import org.slf4j.Logger
import java.io.InputStream

class JsonSchemaException(val errors: Iterable<Error>) : Throwable(errors.joinToString("\n") { it.toString() }) {
	@Serializable
	data class Error(val path: String, val message: String)
}

inline fun <reified T> jsonSchemaLens(schemaSource: InputStream, logger: Logger? = null) =
	jsonSchemaLens<T>(schemaSource.bufferedReader().readText(), logger)

inline fun <reified T> jsonSchemaLens(schemaSource: String, logger: Logger? = null): BiDiBodyLens<T> {
	val schema = JSONSchema.parse(schemaSource)
	return httpBodyLens(contentType = ContentType.APPLICATION_JSON)
		.map(
			{
				// TODO decode to json node and the decode from nodes to avoid parsing the whole string twice?
				logger?.trace("json schema lens decoding string: {}", it)
				val schemaResult = try {
					schema.validateBasic(it)
				} catch (e: JSONException) {
					logger?.debug("json schema failed to validate: ${e.message}")
					throw HttpStatusException(status = Status.BAD_REQUEST, cause = e, message = e.message)
				}
				if (schemaResult.valid) {
					json.decodeFromString<T>(it)
				} else {
					val errors = schemaResult.errors
						?.map { e -> JsonSchemaException.Error(e.keywordLocation, e.error) }
						?: listOf()
					logger?.debug("errors from json schema validation: {}", errors)
					throw JsonSchemaException(errors)
				}
			},
			{
				logger?.trace("json schema lens encoding to string: {}", it)
				json.encodeToString(it)
			}
		)
		.toLens()
}

fun jsonSchemaExceptionFilter(): Filter {
	val lens = Body.auto<List<JsonSchemaException.Error>>().toLens()
	return exceptionFilter<JsonSchemaException> {
		Response(Status.BAD_REQUEST).with(lens of it.errors.toList())
	}
}