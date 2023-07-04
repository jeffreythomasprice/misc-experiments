package com.jeffreythomasprice.experiments

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import org.http4k.core.*
import org.http4k.format.KotlinxSerialization.auto
import org.http4k.routing.bind
import org.http4k.routing.routes
import org.http4k.server.Netty
import org.http4k.server.asServer
import org.slf4j.LoggerFactory

@Serializable
data class HelloWorldRequest(
	@SerialName("foo")
	val stringValue: String,

	@SerialName("bar")
	val intValue: Int
)

@Serializable
data class HelloWorldResponse(
	val message: String
)

fun main() {
	val logger = LoggerFactory.getLogger(object {}.javaClass)
	logger.info("start")

	val helloWorldPost = run {
		val requestLens =
			jsonSchemaLens<HelloWorldRequest>(object {}.javaClass.getResourceAsStream("/schemas/HelloWorldRequest.json")!!)
		val responseLens = Body.auto<HelloWorldResponse>().toLens()
		"/" bind Method.POST to { request ->
			val requestBody = requestLens(request)
			logger.debug("received request: {}", requestBody)
			Response(Status.OK)
				.with(responseLens of HelloWorldResponse("${requestBody.stringValue.reversed()} ${requestBody.intValue + 1}"))
		}
	}

	val server = routes(
		"/" bind Method.GET to {
			Response(Status.I_M_A_TEAPOT).body("I'm a teapot!")
		},
		helloWorldPost
	)
		.withFilter(jsonSchemaExceptionFilter())
		.withFilter(exceptionFilter<HttpStatusException> {
			Response(it.status).body(it.message ?: it.status.description)
		})
		.withFilter(exceptionFilter<Throwable> {
			logger.debug("exception handling request", it)
			throw it
		})
		.asServer(Netty(8000))
		.start()
	logger.info("server started on ${server.port()}")
}
