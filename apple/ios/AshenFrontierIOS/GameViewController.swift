import MetalKit
import UIKit

final class GameViewController: UIViewController {
    private let settings: AppSettings
    private var controller: GameController?
    private var renderer: Renderer?
    private var displayLink: CADisplayLink?
    private let hudLabel = UILabel()
    private let settingsButton = UIButton(type: .system)

    init(settings: AppSettings = .shared) {
        self.settings = settings
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

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
        gameView.preferredFramesPerSecond = settings.targetFrameRate
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
        navigationController?.setNavigationBarHidden(true, animated: false)
        installHud()
        installSettingsButton()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        navigationController?.setNavigationBarHidden(true, animated: animated)
        startHudDisplayLink()
        applyFrameRate()
        refreshHud()
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        stopHudDisplayLink()
    }

    private func startHudDisplayLink() {
        guard displayLink == nil else {
            return
        }

        let displayLink = CADisplayLink(target: self, selector: #selector(refreshHud))
        displayLink.add(to: .main, forMode: .common)
        self.displayLink = displayLink
    }

    private func stopHudDisplayLink() {
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

    private func installSettingsButton() {
        settingsButton.setTitle("Settings", for: .normal)
        settingsButton.titleLabel?.font = .systemFont(ofSize: 14, weight: .semibold)
        settingsButton.tintColor = UIColor(red: 0.86, green: 0.91, blue: 0.82, alpha: 1.0)
        settingsButton.backgroundColor = UIColor(red: 0.08, green: 0.12, blue: 0.10, alpha: 0.72)
        settingsButton.layer.cornerRadius = 8
        settingsButton.translatesAutoresizingMaskIntoConstraints = false
        settingsButton.addTarget(self, action: #selector(openSettings), for: .touchUpInside)
        view.addSubview(settingsButton)

        NSLayoutConstraint.activate([
            settingsButton.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -12),
            settingsButton.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 10),
            settingsButton.widthAnchor.constraint(equalToConstant: 88),
            settingsButton.heightAnchor.constraint(equalToConstant: 36),
        ])
    }

    private func applyFrameRate() {
        let preferredFrameRate = effectiveFrameRate
        let metalView = view as? MTKView
        metalView?.preferredFramesPerSecond = preferredFrameRate
        metalView?.isPaused = false
        displayLink?.preferredFrameRateRange = CAFrameRateRange(
            minimum: 30,
            maximum: Float(preferredFrameRate),
            preferred: Float(preferredFrameRate)
        )
    }

    private var effectiveFrameRate: Int {
        min(settings.targetFrameRate, UIScreen.main.maximumFramesPerSecond)
    }

    @objc private func refreshHud() {
        let gameHudText = controller?.hudText() ?? ""
        hudLabel.text = """
        \(gameHudText)
        Target FPS: \(settings.targetFrameRate)  Effective: \(effectiveFrameRate)
        """
    }

    @objc private func openSettings() {
        let settingsViewController = SettingsViewController(settings: settings)
        settingsViewController.onSettingsChanged = { [weak self] in
            self?.applyFrameRate()
        }
        navigationController?.pushViewController(settingsViewController, animated: true)
    }
}
