package com.jeffreythomasprice.experiments

import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.request.*
import io.ktor.http.*
import kotlinx.serialization.Serializable

// TODO demo, delete me

@Serializable
data class HelloWorldRequest(
	val foo: String,
	val bar: Int
)

@Serializable
data class HelloWorldResponse(
	val message: String
)

sealed class HttpResponseOrError<out T> {
	data class Ok<T>(val value: T) : HttpResponseOrError<T>()
	data class Error(val value: ErrorResponse) : HttpResponseOrError<Nothing>()
}

suspend fun HttpClient.helloWorld(request: HelloWorldRequest): HttpResponseOrError<HelloWorldResponse> {
	val response = request {
		method = HttpMethod.Post
		url {
			protocol = URLProtocol.HTTP
			host = "localhost"
			port = 8001
			path("hello")
		}
		contentType(ContentType.Application.Json)
		setBody(request)
	}
	return if (response.status.isSuccess()) {
		HttpResponseOrError.Ok(response.body())
	} else {
		HttpResponseOrError.Error(response.body())
	}
}