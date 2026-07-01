import AppKit
import MetalKit

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var window: NSWindow?
    private var renderer: Renderer?
    private var world: RustWorld?
    private var hudUpdater: HudUpdater?
    private var hudTimer: Timer?

    func applicationDidFinishLaunching(_ notification: Notification) {
        guard let device = MTLCreateSystemDefaultDevice() else {
            fatalError("Metal is not available on this Mac")
        }

        let world = RustWorld(width: 512, height: 384)
        let controller = GameController(world: world)
        let view = GameView(frame: NSRect(x: 0, y: 0, width: 1280, height: 720), device: device)
        view.colorPixelFormat = .bgra8Unorm
        view.clearColor = MTLClearColor(red: 0.04, green: 0.05, blue: 0.045, alpha: 1.0)
        view.preferredFramesPerSecond = 120
        view.enableSetNeedsDisplay = false
        view.isPaused = false
        view.spawnHorde = {
            controller.spawnHorde()
        }
        view.spawnUnit = { kind in
            controller.spawnUnit(kind: kind)
        }
        view.selectUnit = { point, size in
            controller.selectUnit(at: point, in: size)
        }
        view.moveSelectedUnits = { point, size in
            controller.moveSelectedUnits(to: point, in: size)
        }

        do {
            let renderer = try Renderer(device: device, colorPixelFormat: view.colorPixelFormat, controller: controller)
            view.delegate = renderer
            self.renderer = renderer
        } catch {
            fatalError("failed to create Metal renderer: \(error)")
        }

        let hudLabel = NSTextField(labelWithString: controller.hudText())
        hudLabel.font = .monospacedSystemFont(ofSize: 13, weight: .medium)
        hudLabel.textColor = NSColor(red: 0.86, green: 0.91, blue: 0.82, alpha: 1.0)
        hudLabel.backgroundColor = .clear
        hudLabel.drawsBackground = false
        hudLabel.isBezeled = false
        hudLabel.isEditable = false
        hudLabel.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(hudLabel)
        NSLayoutConstraint.activate([
            hudLabel.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 12),
            hudLabel.topAnchor.constraint(equalTo: view.topAnchor, constant: 10),
        ])
        let hudUpdater = HudUpdater(label: hudLabel, controller: controller)
        hudTimer = Timer.scheduledTimer(
            timeInterval: 0.1,
            target: hudUpdater,
            selector: #selector(HudUpdater.refresh),
            userInfo: nil,
            repeats: true
        )
        self.hudUpdater = hudUpdater

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1280, height: 720),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "Ashen Frontier"
        window.center()
        window.contentView = view
        window.makeKeyAndOrderFront(nil)
        window.makeFirstResponder(view)

        self.world = world
        self.window = window
        NSApp.activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.regular)
app.run()

final class HudUpdater: NSObject {
    private weak var label: NSTextField?
    private let controller: GameController

    init(label: NSTextField, controller: GameController) {
        self.label = label
        self.controller = controller
    }

    @MainActor @objc func refresh() {
        label?.stringValue = controller.hudText()
    }
}
