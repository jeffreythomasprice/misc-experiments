package com.jeffreythomasprice.experiments.models

import kotlinx.serialization.Serializable

@Serializable
data class HelloWorldResponse(
	val message: String
)