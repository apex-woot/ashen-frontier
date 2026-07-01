import AshenFrontierBridge
import simd

enum UnitSpawnKind {
    case worker
    case ranger
    case soldier

    var ffiCode: UInt32 {
        switch self {
        case .worker:
            UInt32(AF_UNIT_KIND_WORKER)
        case .ranger:
            UInt32(AF_UNIT_KIND_RANGER)
        case .soldier:
            UInt32(AF_UNIT_KIND_SOLDIER)
        }
    }
}

final class RustWorld {
    let width: UInt16
    let height: UInt16

    private let handle: OpaquePointer

    init(width: UInt16, height: UInt16) {
        self.width = width
        self.height = height

        guard let handle = af_world_create(width, height) else {
            fatalError("failed to create Rust world")
        }
        self.handle = handle
    }

    deinit {
        af_world_destroy(handle)
    }

    var tick: UInt64 {
        af_world_tick(handle)
    }

    var unitCount: Int {
        af_world_unit_count(handle)
    }

    var enemyCount: Int {
        af_world_enemy_count(handle)
    }

    var size: SIMD2<Float> {
        SIMD2<Float>(Float(width), Float(height))
    }

    func step(_ steps: UInt32) {
        af_world_step(handle, steps)
    }

    func spawnHorde(count: UInt32) {
        af_world_spawn_horde(handle, count)
    }

    func spawnUnit(kind: UnitSpawnKind) -> UInt32 {
        af_world_spawn_unit(handle, kind.ffiCode)
    }

    func moveUnits(ids: [UInt32], destination: SIMD2<Float>) -> UInt32 {
        ids.withUnsafeBufferPointer { buffer in
            af_world_move_units(handle, buffer.baseAddress, buffer.count, destination.x, destination.y)
        }
    }

    func units() -> [AfEntityPosition] {
        readPositions(count: af_world_unit_count(handle), reader: af_world_read_units)
    }

    func enemies() -> [AfEntityPosition] {
        readPositions(count: af_world_enemy_count(handle), reader: af_world_read_enemies)
    }

    func readUnits(into positions: inout [AfEntityPosition]) -> Int {
        readPositions(into: &positions, count: af_world_unit_count(handle), reader: af_world_read_units)
    }

    func readEnemies(into positions: inout [AfEntityPosition]) -> Int {
        readPositions(into: &positions, count: af_world_enemy_count(handle), reader: af_world_read_enemies)
    }

    private func readPositions(
        count: Int,
        reader: (OpaquePointer?, UnsafeMutablePointer<AfEntityPosition>?, Int) -> Int
    ) -> [AfEntityPosition] {
        guard count > 0 else {
            return []
        }

        var positions = Array(repeating: AfEntityPosition(), count: count)
        let written = positions.withUnsafeMutableBufferPointer { buffer in
            reader(handle, buffer.baseAddress, buffer.count)
        }
        return Array(positions.prefix(written))
    }

    private func readPositions(
        into positions: inout [AfEntityPosition],
        count: Int,
        reader: (OpaquePointer?, UnsafeMutablePointer<AfEntityPosition>?, Int) -> Int
    ) -> Int {
        guard count > 0 else {
            positions.removeAll(keepingCapacity: true)
            return 0
        }

        if positions.count < count {
            positions.append(contentsOf: repeatElement(AfEntityPosition(), count: count - positions.count))
        }

        let written = positions.withUnsafeMutableBufferPointer { buffer in
            reader(handle, buffer.baseAddress, count)
        }
        let validCount = min(max(written, 0), positions.count)

        if positions.count != validCount {
            positions.removeSubrange(validCount..<positions.count)
        }
        return validCount
    }
}
