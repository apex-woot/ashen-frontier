import AshenFrontierBridge
import simd

final class RustWorld {
    private let handle: OpaquePointer

    init(width: UInt16, height: UInt16) {
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

    func step(_ steps: UInt32) {
        af_world_step(handle, steps)
    }

    func spawnHorde(count: UInt32) {
        af_world_spawn_horde(handle, count)
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
}
