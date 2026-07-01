import AshenFrontierBridge
import CoreGraphics
import Foundation
import simd

private let selectionRadius: Float = 0.85
private let minimumSelectionHitRadius: Float = 18.0
private let minimumGroupSelectionHitRadius: Float = 44.0
private let minCameraZoom: Float = 1.0
private let maxCameraZoom: Float = 10.0
private let simStepInterval: TimeInterval = 0.1

final class GameController {
    let world: RustWorld
    let worldSize: SIMD2<Float>
    private(set) var selectedUnitIDs: Set<UInt32> = []
    private(set) var focusedEnemyID: UInt32?
    private(set) var fps: Double = 0
    private(set) var lastCommandStatus: UInt32 = UInt32(AF_COMMAND_STATUS_ACCEPTED)
    private let controlHint: String
    private var cameraCenter: SIMD2<Float>
    private var cameraZoom: Float = 1.0
    private var lastFrameTime: TimeInterval?
    private var simAccumulator: TimeInterval = 0.0

    init(
        world: RustWorld,
        controlHint: String = "H=horde  4=worker 5=ranger 6=soldier  Left=select  Right=move",
        initialCameraZoom: Float = 1.0
    ) {
        self.world = world
        self.worldSize = world.size
        self.controlHint = controlHint
        self.cameraCenter = world.size * 0.5
        self.cameraZoom = min(max(initialCameraZoom, minCameraZoom), maxCameraZoom)
    }

    func stepFrame() {
        let delta = updateFPS()
        simAccumulator += min(delta, 0.25)
        while simAccumulator >= simStepInterval {
            world.step(1)
            simAccumulator -= simStepInterval
        }
    }

    func spawnHorde() {
        world.spawnHorde(count: 64)
        focusedEnemyID = enemies().first?.id
        selectedUnitIDs.removeAll()
    }

    func spawnUnit(kind: UnitSpawnKind) {
        let unitID = world.spawnUnit(kind: kind)
        if unitID != 0 {
            selectedUnitIDs = [unitID]
        }
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

    func isFocused(enemyID: UInt32) -> Bool {
        focusedEnemyID == enemyID
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
        let hitRadius = worldSelectionRadius(minimumViewRadius: minimumSelectionHitRadius, in: viewSize)
        let nearest = nearestUnit(to: worldPoint, within: hitRadius)

        if let nearest {
            selectedUnitIDs = [nearest.unit.id]
            focusedEnemyID = nil
        } else {
            selectedUnitIDs.removeAll()
            focusedEnemyID = nil
        }
    }

    func performPrimaryAction(at viewPoint: CGPoint, in viewSize: CGSize) {
        let worldPoint = clampedWorldPosition(for: viewPoint, in: viewSize)
        let hitRadius = worldSelectionRadius(minimumViewRadius: minimumSelectionHitRadius, in: viewSize)

        if let nearest = nearestUnit(to: worldPoint, within: hitRadius) {
            selectedUnitIDs = [nearest.unit.id]
            focusedEnemyID = nil
        } else if let nearest = nearestEnemy(to: worldPoint, within: hitRadius) {
            selectedUnitIDs.removeAll()
            focusedEnemyID = nearest.enemy.id
        } else if selectedUnitIDs.isEmpty {
            selectedUnitIDs.removeAll()
            focusedEnemyID = nil
        } else {
            lastCommandStatus = world.moveUnits(ids: Array(selectedUnitIDs).sorted(), destination: worldPoint)
        }
    }

    func selectUnits(near viewPoint: CGPoint, in viewSize: CGSize) {
        let worldPoint = clampedWorldPosition(for: viewPoint, in: viewSize)
        let hitRadius = worldSelectionRadius(minimumViewRadius: minimumGroupSelectionHitRadius, in: viewSize)
        let hitRadiusSquared = hitRadius * hitRadius
        selectedUnitIDs = Set(
            units()
                .filter { unit in
                    let xDelta = unit.x - worldPoint.x
                    let yDelta = unit.y - worldPoint.y
                    return xDelta * xDelta + yDelta * yDelta <= hitRadiusSquared
                }
                .map(\.id)
        )
        focusedEnemyID = nil
    }

    func selectUnit(id: UInt32) {
        if units().contains(where: { $0.id == id }) {
            selectedUnitIDs = [id]
            focusedEnemyID = nil
        } else {
            selectedUnitIDs.remove(id)
        }
    }

    func selectEnemy(id: UInt32) {
        if enemies().contains(where: { $0.id == id }) {
            focusedEnemyID = id
        } else {
            focusedEnemyID = nil
        }
    }

    func clearSelection() {
        selectedUnitIDs.removeAll()
        focusedEnemyID = nil
    }

    func selectAllUnits() {
        selectedUnitIDs = Set(units().map(\.id))
        focusedEnemyID = nil
    }

    func unitDetails(id: UInt32) -> UnitDetails? {
        guard let unit = units().first(where: { $0.id == id }) else {
            return nil
        }

        return UnitDetails(
            id: unit.id,
            kind: unitKindName(unit.kind),
            position: SIMD2<Float>(unit.x, unit.y),
            health: unit.health
        )
    }

    func focusedUnitDetails() -> UnitDetails? {
        let focusedID = selectedUnitIDs.sorted().first
        guard let focusedID else {
            return nil
        }

        return unitDetails(id: focusedID)
    }

    func enemyDetails(id: UInt32) -> UnitDetails? {
        guard let enemy = enemies().first(where: { $0.id == id }) else {
            return nil
        }

        return UnitDetails(
            id: enemy.id,
            kind: enemyKindName(enemy.kind),
            position: SIMD2<Float>(enemy.x, enemy.y),
            health: enemy.health
        )
    }

    func focusedEnemyDetails() -> UnitDetails? {
        let availableEnemies = enemies()
        let focusedID = focusedEnemyID.flatMap { id in
            availableEnemies.contains(where: { $0.id == id }) ? id : nil
        } ?? availableEnemies.first?.id

        guard let focusedID else {
            return nil
        }

        focusedEnemyID = focusedID
        return enemyDetails(id: focusedID)
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

        cameraCenter -= viewport.viewDeltaToWorld(viewDelta)
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
            return 0
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

    private func nearestUnit(
        to worldPoint: SIMD2<Float>,
        within hitRadius: Float
    ) -> (unit: AfEntityPosition, distanceSquared: Float)? {
        let hitRadiusSquared = hitRadius * hitRadius
        return units()
            .map { unit in
                let xDelta = unit.x - worldPoint.x
                let yDelta = unit.y - worldPoint.y
                return (unit: unit, distanceSquared: xDelta * xDelta + yDelta * yDelta)
            }
            .filter { $0.distanceSquared <= hitRadiusSquared }
            .min { $0.distanceSquared < $1.distanceSquared }
    }

    private func nearestEnemy(
        to worldPoint: SIMD2<Float>,
        within hitRadius: Float
    ) -> (enemy: AfEntityPosition, distanceSquared: Float)? {
        let hitRadiusSquared = hitRadius * hitRadius
        return enemies()
            .map { enemy in
                let xDelta = enemy.x - worldPoint.x
                let yDelta = enemy.y - worldPoint.y
                return (enemy: enemy, distanceSquared: xDelta * xDelta + yDelta * yDelta)
            }
            .filter { $0.distanceSquared <= hitRadiusSquared }
            .min { $0.distanceSquared < $1.distanceSquared }
    }

    private func worldSelectionRadius(minimumViewRadius: Float, in viewSize: CGSize) -> Float {
        let scale = viewport(for: viewSize).scale
        guard scale > 0 else {
            return selectionRadius
        }

        return max(selectionRadius, minimumViewRadius / scale)
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

struct UnitDetails {
    let id: UInt32
    let kind: String
    let position: SIMD2<Float>
    let health: Float
}

private func unitKindName(_ kind: UInt32) -> String {
    switch kind {
    case UInt32(AF_UNIT_KIND_RANGER):
        "Ranger"
    case UInt32(AF_UNIT_KIND_SOLDIER):
        "Soldier"
    default:
        "Worker"
    }
}

private func enemyKindName(_: UInt32) -> String {
    "Infected"
}

struct ViewportTransform {
    let viewSize: CGSize
    let worldSize: SIMD2<Float>
    let center: SIMD2<Float>
    let zoom: Float

    var scale: Float {
        let viewWidth = Float(viewSize.width)
        let viewHeight = Float(viewSize.height)
        let projectedSize = projectedWorldSize

        guard viewWidth > 0, viewHeight > 0, projectedSize.x > 0, projectedSize.y > 0 else {
            return 0
        }

        return min(viewWidth / projectedSize.x, viewHeight / projectedSize.y) * max(zoom, minCameraZoom)
    }

    var visibleWorldSize: SIMD2<Float> {
        guard scale > 0 else {
            return worldSize
        }

        let projectedWidth = Float(viewSize.width) / scale
        let projectedHeight = Float(viewSize.height) / scale
        return SIMD2<Float>(
            min(worldSize.x, (projectedWidth + projectedHeight * 2.0) * 0.5),
            min(worldSize.y, (projectedWidth + projectedHeight * 2.0) * 0.5)
        )
    }

    func viewPointToWorld(_ point: CGPoint) -> SIMD2<Float> {
        guard scale > 0 else {
            return center
        }

        let projectedDelta = SIMD2<Float>(
            (Float(point.x) - Float(viewSize.width) * 0.5) / scale,
            (Float(point.y) - Float(viewSize.height) * 0.5) / scale
        )
        return center + unproject(projectedDelta)
    }

    func viewDeltaToWorld(_ delta: CGSize) -> SIMD2<Float> {
        guard scale > 0 else {
            return SIMD2<Float>(0, 0)
        }

        let projectedDelta = SIMD2<Float>(
            Float(delta.width) / scale,
            Float(delta.height) / scale
        )
        return unproject(projectedDelta)
    }

    func worldToClip(x: Float, y: Float) -> SIMD2<Float> {
        guard viewSize.width > 0, viewSize.height > 0 else {
            return SIMD2<Float>(0, 0)
        }

        let projected = project(SIMD2<Float>(x, y)) - project(center)
        return SIMD2<Float>(
            (projected.x * scale / Float(viewSize.width)) * 2.0,
            (projected.y * scale / Float(viewSize.height)) * 2.0
        )
    }

    func worldHalfSizeToClip(_ halfSize: SIMD2<Float>) -> SIMD2<Float> {
        guard viewSize.width > 0, viewSize.height > 0 else {
            return SIMD2<Float>(0, 0)
        }

        let projectedHalfSize = SIMD2<Float>(
            halfSize.x + halfSize.y,
            (halfSize.x + halfSize.y) * 0.5
        )
        return SIMD2<Float>(
            (projectedHalfSize.x * scale / Float(viewSize.width)) * 2.0,
            (projectedHalfSize.y * scale / Float(viewSize.height)) * 2.0
        )
    }

    func isWorldRectVisible(center: SIMD2<Float>, halfSize: SIMD2<Float>) -> Bool {
        let projectedCenter = project(center) - project(self.center)
        let projectedHalfSize = SIMD2<Float>(
            halfSize.x + halfSize.y,
            (halfSize.x + halfSize.y) * 0.5
        )
        let visibleHalfWidth = Float(viewSize.width) / max(scale, Float.ulpOfOne) * 0.5
        let visibleHalfHeight = Float(viewSize.height) / max(scale, Float.ulpOfOne) * 0.5

        return projectedCenter.x + projectedHalfSize.x >= -visibleHalfWidth
            && projectedCenter.x - projectedHalfSize.x <= visibleHalfWidth
            && projectedCenter.y + projectedHalfSize.y >= -visibleHalfHeight
            && projectedCenter.y - projectedHalfSize.y <= visibleHalfHeight
    }

    private var projectedWorldSize: SIMD2<Float> {
        SIMD2<Float>(
            worldSize.x + worldSize.y,
            (worldSize.x + worldSize.y) * 0.5
        )
    }

    private func project(_ point: SIMD2<Float>) -> SIMD2<Float> {
        SIMD2<Float>(
            point.x - point.y,
            (point.x + point.y) * 0.5
        )
    }

    private func unproject(_ projected: SIMD2<Float>) -> SIMD2<Float> {
        SIMD2<Float>(
            (projected.x + projected.y * 2.0) * 0.5,
            (projected.y * 2.0 - projected.x) * 0.5
        )
    }

}

private func clampedCameraAxis(center: Float, visibleLength: Float, worldLength: Float) -> Float {
    guard visibleLength < worldLength else {
        return worldLength * 0.5
    }

    let halfVisible = visibleLength * 0.5
    return min(max(center, halfVisible), worldLength - halfVisible)
}

func commandStatusLabel(_ status: UInt32) -> String {
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
