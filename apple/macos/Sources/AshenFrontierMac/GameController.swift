import AshenFrontierBridge
import CoreGraphics
import Foundation
import simd

private let gridWidth: Float = 32.0
private let gridHeight: Float = 24.0
private let selectionRadius: Float = 0.85

final class GameController {
    let world: RustWorld
    private(set) var selectedUnitIDs: Set<UInt32> = []
    private(set) var fps: Double = 0
    private(set) var lastCommandStatus: UInt32 = UInt32(AF_COMMAND_STATUS_ACCEPTED)
    private let controlHint: String
    private var lastFrameTime: TimeInterval?

    init(world: RustWorld, controlHint: String = "H=horde  Left=select  Right=move") {
        self.world = world
        self.controlHint = controlHint
    }

    func stepFrame() {
        updateFPS()
        world.step(1)
    }

    func spawnHorde() {
        world.spawnHorde(count: 64)
    }

    func units() -> [AfEntityPosition] {
        world.units()
    }

    func enemies() -> [AfEntityPosition] {
        world.enemies()
    }

    func isSelected(unitID: UInt32) -> Bool {
        selectedUnitIDs.contains(unitID)
    }

    func selectUnit(at viewPoint: CGPoint, in viewSize: CGSize) {
        let worldPoint = worldPosition(for: viewPoint, in: viewSize)
        let nearest = units()
            .map { unit in
                (unit: unit, distance: hypot(unit.x - worldPoint.x, unit.y - worldPoint.y))
            }
            .filter { $0.distance <= selectionRadius }
            .min { $0.distance < $1.distance }

        if let nearest {
            selectedUnitIDs = [nearest.unit.id]
        } else {
            selectedUnitIDs.removeAll()
        }
    }

    func moveSelectedUnits(to viewPoint: CGPoint, in viewSize: CGSize) {
        guard !selectedUnitIDs.isEmpty else {
            lastCommandStatus = UInt32(AF_COMMAND_STATUS_EMPTY_SELECTION)
            return
        }

        let destination = worldPosition(for: viewPoint, in: viewSize)
        lastCommandStatus = world.moveUnits(ids: Array(selectedUnitIDs).sorted(), destination: destination)
    }

    func hudText() -> String {
        """
        FPS: \(String(format: "%.1f", fps))
        Tick: \(world.tick)
        Units: \(world.units().count)
        Enemies: \(world.enemies().count)
        Selected: \(selectedUnitIDs.count)
        Command: \(commandStatusLabel(lastCommandStatus))
        Controls: \(controlHint)
        """
    }

    private func updateFPS() {
        let now = Date.timeIntervalSinceReferenceDate
        defer {
            lastFrameTime = now
        }

        guard let lastFrameTime else {
            return
        }

        let delta = now - lastFrameTime
        guard delta > 0 else {
            return
        }

        let instantFPS = 1.0 / delta
        fps = fps == 0 ? instantFPS : fps * 0.9 + instantFPS * 0.1
    }

    private func worldPosition(for viewPoint: CGPoint, in viewSize: CGSize) -> SIMD2<Float> {
        guard viewSize.width > 0, viewSize.height > 0 else {
            return SIMD2<Float>(0, 0)
        }

        let x = Float(viewPoint.x / viewSize.width) * gridWidth
        let y = Float(viewPoint.y / viewSize.height) * gridHeight
        return SIMD2<Float>(
            min(max(x, 0), gridWidth - Float.ulpOfOne),
            min(max(y, 0), gridHeight - Float.ulpOfOne)
        )
    }
}

func worldToClip(x: Float, y: Float) -> SIMD2<Float> {
    SIMD2<Float>(
        (x / gridWidth) * 2.0 - 1.0,
        (y / gridHeight) * 2.0 - 1.0
    )
}

private func commandStatusLabel(_ status: UInt32) -> String {
    switch status {
    case UInt32(AF_COMMAND_STATUS_ACCEPTED):
        "accepted"
    case UInt32(AF_COMMAND_STATUS_EMPTY_SELECTION):
        "empty selection"
    case UInt32(AF_COMMAND_STATUS_DESTINATION_OUT_OF_BOUNDS):
        "out of bounds"
    case UInt32(AF_COMMAND_STATUS_BLOCKED_DESTINATION):
        "blocked"
    case UInt32(AF_COMMAND_STATUS_NO_PATH):
        "no path"
    case UInt32(AF_COMMAND_STATUS_UNKNOWN_UNIT):
        "unknown unit"
    default:
        "invalid"
    }
}
