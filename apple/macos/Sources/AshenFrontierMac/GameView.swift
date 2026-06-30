import AppKit
import MetalKit

final class GameView: MTKView {
    var spawnHorde: (() -> Void)?
    var selectUnit: ((CGPoint, CGSize) -> Void)?
    var moveSelectedUnits: ((CGPoint, CGSize) -> Void)?

    override var acceptsFirstResponder: Bool {
        true
    }

    override func mouseDown(with event: NSEvent) {
        window?.makeFirstResponder(self)
        selectUnit?(convert(event.locationInWindow, from: nil), bounds.size)
    }

    override func rightMouseDown(with event: NSEvent) {
        window?.makeFirstResponder(self)
        moveSelectedUnits?(convert(event.locationInWindow, from: nil), bounds.size)
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
