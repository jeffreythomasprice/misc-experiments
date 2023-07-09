package com.jeffreythomasprice.experiments

import io.ktor.client.*
import io.ktor.client.plugins.contentnegotiation.*

actual fun createHttpClient(): HttpClient = HttpClient {
	install(ContentNegotiation)
}