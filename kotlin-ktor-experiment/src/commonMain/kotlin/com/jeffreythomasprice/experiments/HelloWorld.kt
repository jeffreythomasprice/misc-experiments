package com.jeffreythomasprice.experiments

import io.ktor.client.*
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

suspend fun HttpClient.helloWorld(request: HelloWorldRequest): HttpResponseOrError<HelloWorldResponse> =
	request {
		method = HttpMethod.Post
		// TODO common url in a config somewhere?
		url {
			protocol = URLProtocol.HTTP
			host = "localhost"
			port = 8001
			path("hello")
		}
		contentType(ContentType.Application.Json)
		setBody(request)
	}
		.toResponseOrError()