import MetalKit
import UIKit

final class GameViewController: UIViewController {
    private var controller: GameController?
    private var renderer: Renderer?
    private var displayLink: CADisplayLink?
    private let hudLabel = UILabel()

    override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
        .portrait
    }

    override var preferredInterfaceOrientationForPresentation: UIInterfaceOrientation {
        .portrait
    }

    override func loadView() {
        guard let device = MTLCreateSystemDefaultDevice() else {
            fatalError("Metal is not available on this device")
        }

        let world = RustWorld(width: 32, height: 24)
        let controller = GameController(
            world: world,
            controlHint: "Tap=select  Long press=move  Drag=pan  Pinch=zoom  Two-finger=horde",
            initialCameraZoom: 2.0
        )
        let gameView = GameView(frame: .zero, device: device)
        gameView.colorPixelFormat = .bgra8Unorm
        gameView.clearColor = MTLClearColor(red: 0.04, green: 0.05, blue: 0.045, alpha: 1.0)
        gameView.enableSetNeedsDisplay = false
        gameView.isPaused = false
        gameView.preferredFramesPerSecond = UIScreen.main.maximumFramesPerSecond
        gameView.spawnHorde = {
            controller.spawnHorde()
        }
        gameView.selectUnit = { point, size in
            controller.selectUnit(at: point, in: size)
        }
        gameView.moveSelectedUnits = { point, size in
            controller.moveSelectedUnits(to: point, in: size)
        }
        gameView.panCamera = { translation, size in
            controller.panCamera(byViewDelta: translation, in: size)
        }
        gameView.zoomCamera = { scale, point, size in
            controller.zoomCamera(by: scale, around: point, in: size)
        }

        do {
            let renderer = try Renderer(device: device, colorPixelFormat: gameView.colorPixelFormat, controller: controller)
            gameView.delegate = renderer
            self.renderer = renderer
        } catch {
            fatalError("failed to create Metal renderer: \(error)")
        }

        self.controller = controller
        view = gameView
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        installHud()

        let displayLink = CADisplayLink(target: self, selector: #selector(refreshHud))
        displayLink.preferredFrameRateRange = CAFrameRateRange(
            minimum: 30,
            maximum: Float(UIScreen.main.maximumFramesPerSecond),
            preferred: Float(UIScreen.main.maximumFramesPerSecond)
        )
        displayLink.add(to: .main, forMode: .common)
        self.displayLink = displayLink
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)
        displayLink?.invalidate()
        displayLink = nil
    }

    private func installHud() {
        hudLabel.translatesAutoresizingMaskIntoConstraints = false
        hudLabel.numberOfLines = 0
        hudLabel.font = .monospacedSystemFont(ofSize: 13, weight: .medium)
        hudLabel.textColor = UIColor(red: 0.86, green: 0.91, blue: 0.82, alpha: 1.0)
        hudLabel.text = controller?.hudText()
        view.addSubview(hudLabel)

        NSLayoutConstraint.activate([
            hudLabel.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 12),
            hudLabel.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 10),
        ])
    }

    @objc private func refreshHud() {
        hudLabel.text = controller?.hudText()
    }
}
