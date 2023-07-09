package com.jeffreythomasprice.experiments

import kotlinx.serialization.Serializable

@Serializable
data class ErrorResponse(
	val messages: List<String>
) {
	constructor(t: Throwable) : this(listOf(t.message ?: "no error message for: ${t::class}"))
}