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
    private let value: JSValue

    fileprivate init(value: JSValue) {
        self.value = value
    }

    func get(name: String) throws -> String? {
        let resultValue = self.value.getAttribute(name)
        if resultValue.isNull || resultValue.isUndefined {
            return nil
        }
        if let result = resultValue.string {
            return result
        }
        throw AttributeError.notAString
    }

    func set(name: String, value: String) {
        _ = self.value.setAttribute(name, value)
    }

    func set(attribute: Attribute) {
        set(name: attribute.name, value: attribute.value)
    }
}

class DOMNode {
    let value: JSObject

    init(value: JSObject) {
        self.value = value
    }

    var attributes: Attributes {
        Attributes(value: value.jsValue)
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

class DOMElement: DOMNode {
    func replaceChildren(child: DOMNode) {
        _ = value.jsValue.replaceChildren(child.value)
    }
}

enum RenderError: Error {
    case failedToCreateTag
    case failedToCreateTextNode
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

        let renderedElement =
            if let result = JSObject.global.document.createElement(tag).object {
                DOMNode(value: result)
            } else {
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

class TextElement: HTML {
    let s: String

    private var renderedElement: DOMNode?

    init(stringLiteral: String) {
        self.s = stringLiteral
    }

    func render() throws -> DOMNode {
        let renderedElement =
            if let result = JSObject.global.document.createTextNode(s).object {
                DOMNode(value: result)
            } else {
                throw RenderError.failedToCreateTextNode
            }
        self.renderedElement = renderedElement
        return renderedElement
    }
}

extension HTML {
    func renderTo(parent: DOMElement) throws {
        parent.replaceChildren(child: try self.render())
    }
}

JavaScriptEventLoop.installGlobalExecutor()

let body = DOMElement(value: JSObject.global.document.body.object!)
let root = Element(tag: "div", attributes: [Attribute(name: "name", value: "foo")], content: [TextElement(stringLiteral: "Hello, World!")])
try root.renderTo(parent: body)
