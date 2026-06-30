import AppKit
import MetalKit

final class GameView: MTKView {
    var spawnHorde: (() -> Void)?

    override var acceptsFirstResponder: Bool {
        true
    }

    override func keyDown(with event: NSEvent) {
        switch event.charactersIgnoringModifiers?.lowercased() {
        case "h":
            spawnHorde?()
        default:
            super.keyDown(with: event)
        }
    }
}
