import AppKit
import MetalKit

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var window: NSWindow?
    private var renderer: Renderer?
    private var world: RustWorld?

    func applicationDidFinishLaunching(_ notification: Notification) {
        guard let device = MTLCreateSystemDefaultDevice() else {
            fatalError("Metal is not available on this Mac")
        }

        let world = RustWorld(width: 32, height: 24)
        let view = GameView(frame: NSRect(x: 0, y: 0, width: 1280, height: 720), device: device)
        view.colorPixelFormat = .bgra8Unorm
        view.clearColor = MTLClearColor(red: 0.04, green: 0.05, blue: 0.045, alpha: 1.0)
        view.preferredFramesPerSecond = 120
        view.enableSetNeedsDisplay = false
        view.isPaused = false
        view.spawnHorde = {
            world.spawnHorde(count: 64)
        }

        do {
            let renderer = try Renderer(device: device, colorPixelFormat: view.colorPixelFormat, world: world)
            view.delegate = renderer
            self.renderer = renderer
        } catch {
            fatalError("failed to create Metal renderer: \(error)")
        }

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
