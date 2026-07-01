import MetalKit
import UIKit

final class GameView: MTKView, UIGestureRecognizerDelegate {
    var primaryAction: ((CGPoint, CGSize) -> Void)?
    var selectUnits: ((CGPoint, CGSize) -> Void)?
    var panCamera: ((CGSize, CGSize) -> Void)?
    var zoomCamera: ((CGFloat, CGPoint, CGSize) -> Void)?

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

        let primaryTap = UITapGestureRecognizer(target: self, action: #selector(handlePrimaryTap(_:)))
        primaryTap.numberOfTouchesRequired = 1
        primaryTap.delegate = self
        addGestureRecognizer(primaryTap)

        let areaSelectPress = UILongPressGestureRecognizer(target: self, action: #selector(handleAreaSelectPress(_:)))
        areaSelectPress.minimumPressDuration = 0.28
        areaSelectPress.allowableMovement = 14
        areaSelectPress.delegate = self
        addGestureRecognizer(areaSelectPress)

        let pan = UIPanGestureRecognizer(target: self, action: #selector(handlePan(_:)))
        pan.delegate = self
        addGestureRecognizer(pan)

        let pinch = UIPinchGestureRecognizer(target: self, action: #selector(handlePinch(_:)))
        pinch.delegate = self
        addGestureRecognizer(pinch)
    }

    @objc private func handlePrimaryTap(_ recognizer: UITapGestureRecognizer) {
        guard recognizer.state == .ended else {
            return
        }

        primaryAction?(touchPointForWorld(from: recognizer), bounds.size)
    }

    @objc private func handleAreaSelectPress(_ recognizer: UILongPressGestureRecognizer) {
        guard recognizer.state == .began else {
            return
        }

        selectUnits?(touchPointForWorld(from: recognizer), bounds.size)
    }

    @objc private func handlePan(_ recognizer: UIPanGestureRecognizer) {
        guard recognizer.state == .changed else {
            return
        }

        panCamera?(translationForWorld(from: recognizer), bounds.size)
        recognizer.setTranslation(.zero, in: self)
    }

    @objc private func handlePinch(_ recognizer: UIPinchGestureRecognizer) {
        guard recognizer.state == .changed else {
            return
        }

        zoomCamera?(recognizer.scale, touchPointForWorld(from: recognizer), bounds.size)
        recognizer.scale = 1.0
    }

    func gestureRecognizer(
        _ gestureRecognizer: UIGestureRecognizer,
        shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer
    ) -> Bool {
        gestureRecognizer is UIPinchGestureRecognizer || otherGestureRecognizer is UIPinchGestureRecognizer
    }

    func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldReceive touch: UITouch) -> Bool {
        touch.view === self
    }

    private func touchPointForWorld(from recognizer: UIGestureRecognizer) -> CGPoint {
        let point = recognizer.location(in: self)
        return CGPoint(x: point.x, y: bounds.height - point.y)
    }

    private func translationForWorld(from recognizer: UIPanGestureRecognizer) -> CGSize {
        let translation = recognizer.translation(in: self)
        return CGSize(width: translation.x, height: -translation.y)
    }
}
