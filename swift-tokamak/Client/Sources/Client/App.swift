import ClientLibrary
import TokamakDOM

@main
struct TokamakApp: App {
  var body: some Scene {
    WindowGroup("Tokamak App") {
      ContentView()
    }
  }
}

struct ContentView: View {
  var body: some View {
    Text("Hello, world!")
  }
}
