package com.jeffreythomasprice.experiments

import io.ktor.client.call.*
import io.ktor.client.statement.*
import io.ktor.http.*
import kotlinx.serialization.Serializable

@Serializable
data class ErrorResponse(
	val messages: List<String>
) {
	constructor(t: Throwable) : this(listOf(t.message ?: "no error message for: ${t::class}"))
}

sealed class HttpResponseOrError<out T> {
	data class Ok<T>(val value: T) : HttpResponseOrError<T>()
	data class Error(val value: ErrorResponse) : HttpResponseOrError<Nothing>()
}

suspend inline fun <reified T> HttpResponse.toResponseOrError() =
	if (status.isSuccess()) {
		HttpResponseOrError.Ok<T>(body())
	} else {
		HttpResponseOrError.Error(body())
	}