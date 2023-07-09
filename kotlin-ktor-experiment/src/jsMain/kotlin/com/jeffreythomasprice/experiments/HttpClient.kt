package com.jeffreythomasprice.experiments

import io.ktor.client.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.serialization.kotlinx.json.*

actual fun createHttpClient(): HttpClient = HttpClient {
	install(ContentNegotiation) {
		json()
	}
}