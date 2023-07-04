package com.jeffreythomasprice.experiments.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class HelloWorldRequest(
	@SerialName("foo")
	val stringValue: String,

	@SerialName("bar")
	val intValue: Int
)