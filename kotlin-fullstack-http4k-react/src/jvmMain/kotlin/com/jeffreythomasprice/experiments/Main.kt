package com.jeffreythomasprice.experiments

import com.jeffreythomasprice.experiments.models.HelloWorldRequest
import com.jeffreythomasprice.experiments.models.HelloWorldResponse
import org.http4k.core.*
import org.http4k.filter.CorsPolicy
import org.http4k.filter.ServerFilters
import org.http4k.format.KotlinxSerialization.auto
import org.http4k.routing.bind
import org.http4k.routing.routes
import org.http4k.server.Netty
import org.http4k.server.asServer
import org.slf4j.LoggerFactory

fun main() {
	val logger = LoggerFactory.getLogger(object {}.javaClass)
	logger.info("start")

	val helloWorldPost = run {
		val requestLens =
			jsonSchemaLens<HelloWorldRequest>(object {}.javaClass.getResourceAsStream("/schemas/HelloWorldRequest.json")!!)
		val responseLens = Body.auto<HelloWorldResponse>().toLens();
		{ request: Request ->
			val requestBody = requestLens(request)
			logger.debug("received request: {}", requestBody)
			Response(Status.OK)
				.with(responseLens of HelloWorldResponse("${requestBody.stringValue.reversed()} ${requestBody.intValue + 1}"))
		}
	}

	val server = ServerFilters.Cors(CorsPolicy.UnsafeGlobalPermissive)
		.then(
			jsonSchemaExceptionFilter()
		)
		.then(exceptionFilter<HttpStatusException> {
			Response(it.status).body(it.message ?: it.status.description)
		})
		.then(exceptionFilter<Throwable> {
			logger.debug("exception handling request", it)
			throw it
		})
		.then(
			routes(
				"/hello" bind Method.POST to helloWorldPost
			)
		)
		.asServer(Netty(8001))
		.start()
	logger.info("server started on ${server.port()}")
}
