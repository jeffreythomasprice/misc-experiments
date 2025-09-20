import JavaScriptKit

let document = JSObject.global.document

var count = 0
var div = document.createElement("div")
_ = document.body.appendChild(div)
@MainActor
func updateCount() {
    div.innerText = "Count: \(count)".jsValue
}
updateCount()

// Handle events with Swift closures
var button = document.createElement("button")
button.innerText = "Click me"
button.onclick = .object(
    JSClosure { _ in
        count += 1
        updateCount()
        return .undefined
    })
_ = document.body.appendChild(button)
