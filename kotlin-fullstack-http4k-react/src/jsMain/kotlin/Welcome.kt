import com.jeffreythomasprice.experiments.HelloWorldClientService
import com.jeffreythomasprice.experiments.models.HelloWorldRequest
import csstype.Color
import csstype.em
import csstype.px
import emotion.css.ClassName
import emotion.react.css
import io.ktor.client.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.serialization.kotlinx.json.*
import io.ktor.utils.io.core.*
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.launch
import react.FC
import react.Props
import react.dom.html.InputType
import react.dom.html.ReactHTML.div
import react.dom.html.ReactHTML.input
import react.useEffectOnce
import react.useState

external interface WelcomeProps : Props {
	var name: String
}

val Welcome = FC<WelcomeProps> { props ->
	var name by useState(props.name)

	val colors = ClassName {
		backgroundColor = Color("orange")
		color = Color("blue")
		padding = 5.px
	}
	val bigFont = ClassName {
		fontSize = 2.em
	}

	useEffectOnce {
		MainScope().launch {
			// TODO make http client factory func, see https://ktor.io/docs/http-client-engines.html#create
			HttpClient() {
				install(ContentNegotiation) {
					json()
				}
			}
				.use { client ->
					val result = HelloWorldClientService.helloWorld(client, HelloWorldRequest("foo", 42))
					console.log("result = $result")
				}
		}
	}

	div {
		css(colors, bigFont) {}
		+"Hello, $name"
	}
	input {
		css {
			marginTop = 5.px
			marginBottom = 5.px
			fontSize = 14.px
		}
		type = InputType.text
		value = name
		onChange = { event ->
			name = event.target.value
		}
	}
}