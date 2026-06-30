import AshenFrontierBridge
import CoreGraphics
import Foundation
import simd

private let selectionRadius: Float = 0.85
private let groupSelectionRadius: Float = 2.25
private let selectionRadiusSquared = selectionRadius * selectionRadius
private let groupSelectionRadiusSquared = groupSelectionRadius * groupSelectionRadius
private let minCameraZoom: Float = 0.75
private let maxCameraZoom: Float = 24.0
private let simulationStepInterval: TimeInterval = 1.0 / 60.0
private let maxSimulationStepsPerFrame = 2

final class GameController {
    let world: RustWorld
    let worldSize: SIMD2<Float>
    private(set) var selectedUnitIDs: Set<UInt32> = []
    private(set) var fps: Double = 0
    private(set) var lastCommandStatus: UInt32 = UInt32(AF_COMMAND_STATUS_ACCEPTED)
    private let controlHint: String
    private var cameraCenter: SIMD2<Float>
    private var cameraZoom: Float = 1.0
    private var lastFrameTime: TimeInterval?
    private var simulationAccumulator: TimeInterval = 0

    init(
        world: RustWorld,
        controlHint: String = "H=horde  Left=select  Right=move",
        initialCameraZoom: Float = 1.0
    ) {
        self.world = world
        self.worldSize = world.size
        self.controlHint = controlHint
        self.cameraCenter = world.size * 0.5
        self.cameraZoom = min(max(initialCameraZoom, minCameraZoom), maxCameraZoom)
    }

    func stepFrame() {
        let frameDelta = updateFPS()
        simulationAccumulator += min(frameDelta, simulationStepInterval * TimeInterval(maxSimulationStepsPerFrame))

        var stepCount = 0
        while simulationAccumulator >= simulationStepInterval && stepCount < maxSimulationStepsPerFrame {
            world.step(1)
            simulationAccumulator -= simulationStepInterval
            stepCount += 1
        }
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

    func readUnits(into positions: inout [AfEntityPosition]) -> Int {
        world.readUnits(into: &positions)
    }

    func readEnemies(into positions: inout [AfEntityPosition]) -> Int {
        world.readEnemies(into: &positions)
    }

    func isSelected(unitID: UInt32) -> Bool {
        selectedUnitIDs.contains(unitID)
    }

    func viewport(for viewSize: CGSize) -> ViewportTransform {
        ViewportTransform(
            viewSize: viewSize,
            worldSize: worldSize,
            center: cameraCenter,
            zoom: cameraZoom
        )
    }

    func selectUnit(at viewPoint: CGPoint, in viewSize: CGSize) {
        let worldPoint = clampedWorldPosition(for: viewPoint, in: viewSize)
        let nearest = units()
            .map { unit in
                let xDelta = unit.x - worldPoint.x
                let yDelta = unit.y - worldPoint.y
                return (unit: unit, distanceSquared: xDelta * xDelta + yDelta * yDelta)
            }
            .filter { $0.distanceSquared <= selectionRadiusSquared }
            .min { $0.distanceSquared < $1.distanceSquared }

        if let nearest {
            selectedUnitIDs = [nearest.unit.id]
        } else {
            selectedUnitIDs.removeAll()
        }
    }

    func selectUnits(near viewPoint: CGPoint, in viewSize: CGSize) {
        let worldPoint = clampedWorldPosition(for: viewPoint, in: viewSize)
        let selectedIDs = units()
            .filter { unit in
                let xDelta = unit.x - worldPoint.x
                let yDelta = unit.y - worldPoint.y
                return xDelta * xDelta + yDelta * yDelta <= groupSelectionRadiusSquared
            }
            .map(\.id)

        selectedUnitIDs = Set(selectedIDs)
    }

    func selectAllUnits() {
        selectedUnitIDs = Set(units().map(\.id))
    }

    func moveSelectedUnits(to viewPoint: CGPoint, in viewSize: CGSize) {
        guard !selectedUnitIDs.isEmpty else {
            lastCommandStatus = UInt32(AF_COMMAND_STATUS_EMPTY_SELECTION)
            return
        }

        let destination = clampedWorldPosition(for: viewPoint, in: viewSize)
        lastCommandStatus = world.moveUnits(ids: Array(selectedUnitIDs).sorted(), destination: destination)
    }

    func panCamera(byViewDelta viewDelta: CGSize, in viewSize: CGSize) {
        let viewport = viewport(for: viewSize)
        guard viewport.scale > 0 else {
            return
        }

        cameraCenter.x -= Float(viewDelta.width) / viewport.scale
        cameraCenter.y -= Float(viewDelta.height) / viewport.scale
        clampCameraCenter(for: viewSize)
    }

    func zoomCamera(by scaleFactor: CGFloat, around viewPoint: CGPoint, in viewSize: CGSize) {
        guard scaleFactor.isFinite, scaleFactor > 0 else {
            return
        }

        let beforeZoom = viewport(for: viewSize).viewPointToWorld(viewPoint)
        cameraZoom = min(max(cameraZoom * Float(scaleFactor), minCameraZoom), maxCameraZoom)
        let afterZoom = viewport(for: viewSize).viewPointToWorld(viewPoint)
        cameraCenter += beforeZoom - afterZoom
        clampCameraCenter(for: viewSize)
    }

    func hudText() -> String {
        """
        FPS: \(String(format: "%.1f", fps))
        Tick: \(world.tick)
        Units: \(world.unitCount)
        Enemies: \(world.enemyCount)
        Selected: \(selectedUnitIDs.count)
        Command: \(commandStatusLabel(lastCommandStatus))
        Controls: \(controlHint)
        """
    }

    private func updateFPS() -> TimeInterval {
        let now = Date.timeIntervalSinceReferenceDate
        defer {
            lastFrameTime = now
        }

        guard let lastFrameTime else {
            return simulationStepInterval
        }

        let delta = now - lastFrameTime
        guard delta > 0 else {
            return 0
        }

        let instantFPS = 1.0 / delta
        fps = fps == 0 ? instantFPS : fps * 0.9 + instantFPS * 0.1
        return delta
    }

    private func clampedWorldPosition(for viewPoint: CGPoint, in viewSize: CGSize) -> SIMD2<Float> {
        let point = viewport(for: viewSize).viewPointToWorld(viewPoint)
        return SIMD2<Float>(
            min(max(point.x, 0), worldSize.x - Float.ulpOfOne),
            min(max(point.y, 0), worldSize.y - Float.ulpOfOne)
        )
    }

    private func clampCameraCenter(for viewSize: CGSize) {
        let viewport = viewport(for: viewSize)
        let visible = viewport.visibleWorldSize
        cameraCenter.x = clampedCameraAxis(
            center: cameraCenter.x,
            visibleLength: visible.x,
            worldLength: worldSize.x
        )
        cameraCenter.y = clampedCameraAxis(
            center: cameraCenter.y,
            visibleLength: visible.y,
            worldLength: worldSize.y
        )
    }
}

struct ViewportTransform {
    let viewSize: CGSize
    let worldSize: SIMD2<Float>
    let center: SIMD2<Float>
    let zoom: Float

    var scale: Float {
        let viewWidth = Float(viewSize.width)
        let viewHeight = Float(viewSize.height)
        let worldWidth = worldSize.x
        let worldHeight = worldSize.y

        guard viewWidth > 0, viewHeight > 0, worldWidth > 0, worldHeight > 0 else {
            return 0
        }

        return min(viewWidth / worldWidth, viewHeight / worldHeight) * max(zoom, minCameraZoom)
    }

    var visibleWorldSize: SIMD2<Float> {
        guard scale > 0 else {
            return worldSize
        }

        return SIMD2<Float>(
            Float(viewSize.width) / scale,
            Float(viewSize.height) / scale
        )
    }

    func viewPointToWorld(_ point: CGPoint) -> SIMD2<Float> {
        guard scale > 0 else {
            return center
        }

        return SIMD2<Float>(
            center.x + (Float(point.x) - Float(viewSize.width) * 0.5) / scale,
            center.y + (Float(point.y) - Float(viewSize.height) * 0.5) / scale
        )
    }

    func worldToClip(x: Float, y: Float) -> SIMD2<Float> {
        guard viewSize.width > 0, viewSize.height > 0 else {
            return SIMD2<Float>(0, 0)
        }

        return SIMD2<Float>(
            ((x - center.x) * scale / Float(viewSize.width)) * 2.0,
            ((y - center.y) * scale / Float(viewSize.height)) * 2.0
        )
    }

    func worldHalfSizeToClip(_ halfSize: SIMD2<Float>) -> SIMD2<Float> {
        guard viewSize.width > 0, viewSize.height > 0 else {
            return SIMD2<Float>(0, 0)
        }

        return SIMD2<Float>(
            (halfSize.x * scale / Float(viewSize.width)) * 2.0,
            (halfSize.y * scale / Float(viewSize.height)) * 2.0
        )
    }

    func isWorldRectVisible(center rectCenter: SIMD2<Float>, halfSize: SIMD2<Float>) -> Bool {
        let visible = visibleWorldSize
        let minVisible = center - visible * 0.5
        let maxVisible = center + visible * 0.5
        let minRect = rectCenter - halfSize
        let maxRect = rectCenter + halfSize

        return maxRect.x >= minVisible.x
            && minRect.x <= maxVisible.x
            && maxRect.y >= minVisible.y
            && minRect.y <= maxVisible.y
    }
}

private func clampedCameraAxis(center: Float, visibleLength: Float, worldLength: Float) -> Float {
    guard visibleLength < worldLength else {
        return worldLength * 0.5
    }

    let halfVisible = visibleLength * 0.5
    return min(max(center, halfVisible), worldLength - halfVisible)
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
