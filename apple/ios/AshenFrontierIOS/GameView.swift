import MetalKit
import UIKit

final class GameView: MTKView {
    var spawnHorde: (() -> Void)?
    var selectUnit: ((CGPoint, CGSize) -> Void)?
    var moveSelectedUnits: ((CGPoint, CGSize) -> Void)?

    override init(frame frameRect: CGRect, device: MTLDevice?) {
        super.init(frame: frameRect, device: device)
        installGestures()
    }

    required init(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func installGestures() {
        isUserInteractionEnabled = true
        isMultipleTouchEnabled = true

        let selectTap = UITapGestureRecognizer(target: self, action: #selector(handleSelectTap(_:)))
        selectTap.numberOfTouchesRequired = 1
        addGestureRecognizer(selectTap)

        let hordeTap = UITapGestureRecognizer(target: self, action: #selector(handleHordeTap(_:)))
        hordeTap.numberOfTouchesRequired = 2
        addGestureRecognizer(hordeTap)
        selectTap.require(toFail: hordeTap)

        let movePress = UILongPressGestureRecognizer(target: self, action: #selector(handleMovePress(_:)))
        movePress.minimumPressDuration = 0.25
        addGestureRecognizer(movePress)
    }

    @objc private func handleSelectTap(_ recognizer: UITapGestureRecognizer) {
        guard recognizer.state == .ended else {
            return
        }

        selectUnit?(touchPointForWorld(from: recognizer), bounds.size)
    }

    @objc private func handleHordeTap(_ recognizer: UITapGestureRecognizer) {
        guard recognizer.state == .ended else {
            return
        }

        spawnHorde?()
    }

    @objc private func handleMovePress(_ recognizer: UILongPressGestureRecognizer) {
        guard recognizer.state == .began else {
            return
        }

        moveSelectedUnits?(touchPointForWorld(from: recognizer), bounds.size)
    }

    private func touchPointForWorld(from recognizer: UIGestureRecognizer) -> CGPoint {
        let point = recognizer.location(in: self)
        return CGPoint(x: point.x, y: bounds.height - point.y)
    }
}
