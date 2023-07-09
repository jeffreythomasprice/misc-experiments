package com.jeffreythomasprice.experiments

import kotlinx.browser.document
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.launch
import react.FC
import react.Props
import react.create
import react.dom.client.createRoot
import react.dom.html.ReactHTML.div
import react.dom.html.ReactHTML.h1
import react.useState

fun main() {
	val container = document.createElement("div")
	document.body!!.appendChild(container)

	MainScope().launch {
		createHttpClient().helloWorld(
			HelloWorldRequest(
				foo = "a",
				bar = 42
			)
		)
			.let {
				when (it) {
					is HttpResponseOrError.Ok -> console.log("success: ${it.value}")
					is HttpResponseOrError.Error -> {
						val errorMessages = it.value.messages.joinToString("\n")
						console.log("failure:\n$errorMessages")
					}
				}
			}
	}

	createRoot(container)
		.render(HelloWorld.create {
			value = "Hello, World!"
		})
}

external interface HelloWorldProps : Props {
	var value: String
}

val HelloWorld = FC<HelloWorldProps> { props ->
	var value by useState(props.value)

	div {
		h1 {
			+value
		}
	}
}