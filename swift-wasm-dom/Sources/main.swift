import JavaScriptEventLoop
import JavaScriptKit

enum AttributeError: Error {
    case notAString
}

struct Attribute {
    let name: String
    let value: String
}

struct Attributes {
    private let value: JSObject

    fileprivate init(value: JSObject) {
        self.value = value
    }

    func get(name: String) throws -> String? {
        let resultValue = self.value[dynamicMember: name]
        if resultValue.isNull || resultValue.isUndefined {
            return nil
        }
        if let result = resultValue.string {
            return result
        }
        throw AttributeError.notAString
    }

    func set(name: String, value: String) {
        self.value[dynamicMember: name] = JSValue(stringLiteral: value)
    }

    func set(attribute: Attribute) {
        set(name: attribute.name, value: attribute.value)
    }
}

struct DOMNode {
    let value: JSObject

    var attributes: Attributes {
        Attributes(value: value)
    }

    var parentNode: DOMNode? {
        let result = value.parentNode
        print("TODO parentNode = \(result)")
        return nil
    }

    func appendChild(child: DOMNode) {
        _ = value.jsValue.appendChild(child.value)
    }

    func replaceChild(newChild: DOMNode, oldChild: DOMNode) {
        _ = value.jsValue.replaceChild(newChild.value, oldChild.value)
    }
}

func createElement(tag: String) -> DOMNode? {
    if let result = JSObject.global.document.createElement(tag).object {
        DOMNode(value: result)
    } else {
        nil
    }
}

enum RenderError: Error {
    case failedToCreateTag
}

protocol HTML {
    func render() throws -> DOMNode
}

class Element: HTML {
    let tag: String
    let attributes: [Attribute]
    let content: [HTML]

    private var renderedElement: DOMNode?

    init(tag: String, attributes: [Attribute], content: [HTML]) {
        self.tag = tag
        self.attributes = attributes
        self.content = content
    }

    func render() throws -> DOMNode {
        // TODO only re-render if dirty?

        guard let renderedElement = createElement(tag: tag) else {
            throw RenderError.failedToCreateTag
        }

        for a in attributes {
            renderedElement.attributes.set(attribute: a)
        }

        for c in content {
            // TODO only re-render if dirty?
            renderedElement.appendChild(child: try c.render())
        }

        if let current = self.renderedElement {
            if let parent = current.parentNode {
                parent.replaceChild(newChild: renderedElement, oldChild: current)
            }
        }

        self.renderedElement = renderedElement
        return renderedElement
    }
}

JavaScriptEventLoop.installGlobalExecutor()

let root = Element(tag: "div", attributes: [Attribute(name: "name", value: "foo")], content: [])
// var root = JSObject.global.document.createElement("div")
// root.innerText = "Hello, World!"
_ = JSObject.global.document.body.replaceChildren(root.)
