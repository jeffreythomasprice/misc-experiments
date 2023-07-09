package com.jeffreythomasprice.experiments

import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import io.ktor.server.application.*
import io.ktor.server.engine.*
import io.ktor.server.netty.*
import io.ktor.server.plugins.contentnegotiation.*
import io.ktor.server.plugins.cors.routing.*
import io.ktor.server.plugins.statuspages.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import org.slf4j.LoggerFactory

fun main() {
	val logger = LoggerFactory.getLogger(object {}.javaClass)

	embeddedServer(
		Netty,

		host = "127.0.0.1",
		port = 8001,

		module = {
			install(CORS) {
				anyHost()
				allowOrigins { true }
				allowHeaders { true }
				allowNonSimpleContentTypes = true
				allowMethod(HttpMethod.Get)
				allowMethod(HttpMethod.Head)
				allowMethod(HttpMethod.Options)
				allowMethod(HttpMethod.Put)
				allowMethod(HttpMethod.Post)
				allowMethod(HttpMethod.Patch)
				allowMethod(HttpMethod.Delete)
			}

			install(ContentNegotiation) {
				json()
			}

			install(StatusPages) {
				jsonValidator()

				exception<Throwable> { call, cause ->
					logger.error("unhandled error", cause)
					val response = ErrorResponse(cause)
					if (call.request.contentType() == ContentType.Application.Json) {
						call.respond(
							status = HttpStatusCode.InternalServerError,
							message = response
						)
					} else {
						call.respondText(
							status = HttpStatusCode.InternalServerError,
							text = response.messages.joinToString("\n")
						)
					}
				}
			}

			routing {
				get("/") {
					call.respondText("Hello, World!")
				}

				get("/simulated-error") {
					throw Exception("this is a test error")
				}

				JSONSchemaValidator.fromResource<HelloWorldRequest>("/schemas/HelloWorldRequest.json")?.let { schema ->
					post("/hello") {
						val request = call.receive(schema)
						call.respond(HelloWorldResponse("${request.foo.reversed()} ${request.bar + 1}"))
					}
				}
			}
		}
	)
		.let {
			logger.debug("starting server ${it.environment.config.host} ${it.environment.config.port}")
			it
		}
		.start(wait = true)
}
