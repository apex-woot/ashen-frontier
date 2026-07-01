import AshenFrontierBridge
import Metal
import MetalKit
import simd

struct Vertex {
    var position: SIMD2<Float>
    var worldPosition: SIMD2<Float>
    var color: SIMD4<Float>
    var material: Float
}

final class Renderer: NSObject, MTKViewDelegate {
    private static let minimumVertexBufferLength = 4096

    private let device: MTLDevice
    private let controller: GameController
    private let commandQueue: MTLCommandQueue
    private let pipelineState: MTLRenderPipelineState
    private var vertexBuffer: MTLBuffer?
    private var vertexBufferLength = 0
    private var sceneVertices: [Vertex] = []
    private var unitPositions: [AfEntityPosition] = []
    private var enemyPositions: [AfEntityPosition] = []

    init(device: MTLDevice, colorPixelFormat: MTLPixelFormat, controller: GameController) throws {
        guard let commandQueue = device.makeCommandQueue() else {
            throw RendererError.missingCommandQueue
        }

        self.device = device
        self.controller = controller
        self.commandQueue = commandQueue
        self.pipelineState = try Self.makePipelineState(device: device, pixelFormat: colorPixelFormat)
        super.init()
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        guard let descriptor = view.currentRenderPassDescriptor,
              let drawable = view.currentDrawable,
              let commandBuffer = commandQueue.makeCommandBuffer(),
              let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: descriptor) else {
            return
        }

        controller.stepFrame()
        let viewport = controller.viewport(for: view.bounds.size)
        sceneVertices.removeAll(keepingCapacity: true)
        makeSceneVertices(viewport: viewport, into: &sceneVertices)
        guard let vertexBuffer = prepareVertexBuffer(for: sceneVertices) else {
            return
        }

        encoder.setRenderPipelineState(pipelineState)
        encoder.setVertexBuffer(vertexBuffer, offset: 0, index: 0)
        encoder.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: sceneVertices.count)
        encoder.endEncoding()
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }

    private static func makePipelineState(
        device: MTLDevice,
        pixelFormat: MTLPixelFormat
    ) throws -> MTLRenderPipelineState {
        let library = try makeShaderLibrary(device: device)

        let descriptor = MTLRenderPipelineDescriptor()
        descriptor.vertexFunction = library.makeFunction(name: "vertex_main")
        descriptor.fragmentFunction = library.makeFunction(name: "fragment_main")
        descriptor.colorAttachments[0].pixelFormat = pixelFormat
        descriptor.vertexDescriptor = makeVertexDescriptor()

        return try device.makeRenderPipelineState(descriptor: descriptor)
    }

    private static func makeShaderLibrary(device: MTLDevice) throws -> MTLLibrary {
#if SWIFT_PACKAGE
        guard let shaderURL = Bundle.module.url(
            forResource: "AshenShaders",
            withExtension: "metal",
            subdirectory: "Shaders"
        ) else {
            throw RendererError.missingShader
        }

        let shaderSource = try String(contentsOf: shaderURL)
        return try device.makeLibrary(source: shaderSource, options: nil)
#else
        guard let library = device.makeDefaultLibrary() else {
            throw RendererError.missingShader
        }
        return library
#endif
    }

    private static func makeVertexDescriptor() -> MTLVertexDescriptor {
        let descriptor = MTLVertexDescriptor()
        descriptor.attributes[0].format = .float2
        descriptor.attributes[0].offset = 0
        descriptor.attributes[0].bufferIndex = 0
        descriptor.attributes[1].format = .float2
        descriptor.attributes[1].offset = MemoryLayout<SIMD2<Float>>.stride
        descriptor.attributes[1].bufferIndex = 0
        descriptor.attributes[2].format = .float4
        descriptor.attributes[2].offset = MemoryLayout<SIMD2<Float>>.stride * 2
        descriptor.attributes[2].bufferIndex = 0
        descriptor.attributes[3].format = .float
        descriptor.attributes[3].offset = MemoryLayout<SIMD2<Float>>.stride * 2 + MemoryLayout<SIMD4<Float>>.stride
        descriptor.attributes[3].bufferIndex = 0
        descriptor.layouts[0].stride = MemoryLayout<Vertex>.stride
        descriptor.layouts[0].stepFunction = .perVertex
        return descriptor
    }

    private func prepareVertexBuffer(for vertices: [Vertex]) -> MTLBuffer? {
        let byteCount = vertices.count * MemoryLayout<Vertex>.stride
        guard byteCount > 0 else {
            return nil
        }

        guard let buffer = vertexBuffer(minimumLength: byteCount) else {
            return nil
        }

        vertices.withUnsafeBytes { bytes in
            guard let baseAddress = bytes.baseAddress else {
                return
            }

            buffer.contents().copyMemory(from: baseAddress, byteCount: byteCount)
        }
        return buffer
    }

    private func vertexBuffer(minimumLength: Int) -> MTLBuffer? {
        if let vertexBuffer, vertexBufferLength >= minimumLength {
            return vertexBuffer
        }

        var newLength = max(Self.minimumVertexBufferLength, vertexBufferLength)
        while newLength < minimumLength {
            newLength *= 2
        }

        guard let newBuffer = device.makeBuffer(length: newLength, options: .storageModeShared) else {
            return nil
        }

        vertexBuffer = newBuffer
        vertexBufferLength = newLength
        return newBuffer
    }

    private func makeSceneVertices(viewport: ViewportTransform, into vertices: inout [Vertex]) {
        let unitCount = controller.readUnits(into: &unitPositions)
        let enemyCount = controller.readEnemies(into: &enemyPositions)
        vertices.reserveCapacity(6 + ((unitCount + enemyCount) * 6))

        appendTerrainQuad(
            worldSize: controller.worldSize,
            viewport: viewport,
            to: &vertices
        )

        for unit in unitPositions {
            let worldCenter = SIMD2<Float>(unit.x, unit.y)
            let halfSize = SIMD2<Float>(0.29, 0.29)
            guard viewport.isWorldRectVisible(center: worldCenter, halfSize: halfSize) else {
                continue
            }

            appendQuad(
                center: viewport.worldToClip(x: unit.x, y: unit.y),
                halfSize: viewport.worldHalfSizeToClip(halfSize),
                worldCenter: worldCenter,
                color: controller.isSelected(unitID: unit.id)
                    ? SIMD4<Float>(0.95, 0.86, 0.34, 1.0)
                    : SIMD4<Float>(0.76, 0.82, 0.64, 1.0),
                to: &vertices
            )
        }

        for enemy in enemyPositions {
            let worldCenter = SIMD2<Float>(enemy.x, enemy.y)
            let halfSize = SIMD2<Float>(0.26, 0.26)
            guard viewport.isWorldRectVisible(center: worldCenter, halfSize: halfSize) else {
                continue
            }

            appendQuad(
                center: viewport.worldToClip(x: enemy.x, y: enemy.y),
                halfSize: viewport.worldHalfSizeToClip(halfSize),
                worldCenter: worldCenter,
                color: controller.isFocused(enemyID: enemy.id)
                    ? SIMD4<Float>(1.00, 0.48, 0.20, 1.0)
                    : SIMD4<Float>(0.78, 0.18, 0.16, 1.0),
                to: &vertices
            )
        }
    }
}

private func appendTerrainQuad(
    worldSize: SIMD2<Float>,
    viewport: ViewportTransform,
    to vertices: inout [Vertex]
) {
    let bottomLeftWorld = SIMD2<Float>(0, 0)
    let bottomRightWorld = SIMD2<Float>(worldSize.x, 0)
    let topLeftWorld = SIMD2<Float>(0, worldSize.y)
    let topRightWorld = worldSize

    let bottomLeft = viewport.worldToClip(x: bottomLeftWorld.x, y: bottomLeftWorld.y)
    let bottomRight = viewport.worldToClip(x: bottomRightWorld.x, y: bottomRightWorld.y)
    let topLeft = viewport.worldToClip(x: topLeftWorld.x, y: topLeftWorld.y)
    let topRight = viewport.worldToClip(x: topRightWorld.x, y: topRightWorld.y)
    let color = SIMD4<Float>(0.14, 0.22, 0.18, 1.0)

    vertices.append(Vertex(position: topLeft, worldPosition: topLeftWorld, color: color, material: 0))
    vertices.append(Vertex(position: bottomLeft, worldPosition: bottomLeftWorld, color: color, material: 0))
    vertices.append(Vertex(position: topRight, worldPosition: topRightWorld, color: color, material: 0))
    vertices.append(Vertex(position: topRight, worldPosition: topRightWorld, color: color, material: 0))
    vertices.append(Vertex(position: bottomLeft, worldPosition: bottomLeftWorld, color: color, material: 0))
    vertices.append(Vertex(position: bottomRight, worldPosition: bottomRightWorld, color: color, material: 0))
}

private func appendQuad(
    center: SIMD2<Float>,
    halfSize: SIMD2<Float>,
    worldCenter: SIMD2<Float>,
    color: SIMD4<Float>,
    to vertices: inout [Vertex]
) {
    let topLeft = SIMD2<Float>(center.x - halfSize.x, center.y + halfSize.y)
    let topRight = SIMD2<Float>(center.x + halfSize.x, center.y + halfSize.y)
    let bottomLeft = SIMD2<Float>(center.x - halfSize.x, center.y - halfSize.y)
    let bottomRight = SIMD2<Float>(center.x + halfSize.x, center.y - halfSize.y)

    vertices.append(Vertex(position: topLeft, worldPosition: worldCenter, color: color, material: 1))
    vertices.append(Vertex(position: bottomLeft, worldPosition: worldCenter, color: color, material: 1))
    vertices.append(Vertex(position: topRight, worldPosition: worldCenter, color: color, material: 1))
    vertices.append(Vertex(position: topRight, worldPosition: worldCenter, color: color, material: 1))
    vertices.append(Vertex(position: bottomLeft, worldPosition: worldCenter, color: color, material: 1))
    vertices.append(Vertex(position: bottomRight, worldPosition: worldCenter, color: color, material: 1))
}

enum RendererError: Error {
    case missingCommandQueue
    case missingShader
}
