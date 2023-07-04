package com.jeffreythomasprice.experiments

import com.jeffreythomasprice.experiments.models.HelloWorldRequest
import com.jeffreythomasprice.experiments.models.HelloWorldResponse
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.request.*
import io.ktor.http.*

object HelloWorldClientService {
	suspend fun helloWorld(client: HttpClient, request: HelloWorldRequest): HelloWorldResponse {
		// TODO url should be determined by whether we're running in dev mode or not
		val response = client.post("http://localhost:8001/hello") {
			contentType(ContentType.Application.Json)
			setBody(request)
		}
		// TODO should be able to parse error responses differently
		return response.body<HelloWorldResponse>()
	}
}